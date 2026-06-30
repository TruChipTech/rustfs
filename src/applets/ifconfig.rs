/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ifconfig — configure network interfaces

use std::ffi::CString;
use std::io;
use std::mem;

extern "C" {
    fn inet_pton(af: libc::c_int, src: *const libc::c_char, dst: *mut libc::c_void) -> libc::c_int;
}

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        return show_all_interfaces();
    }

    let iface = &args[0];

    if args.len() == 1 {
        return show_interface(iface);
    }

    // Parse configuration commands
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "up"
                if set_interface_flags(iface, true) != 0 => { return 1; }
            "down"
                if set_interface_flags(iface, false) != 0 => { return 1; }
            "netmask" => {
                i += 1;
                if i < args.len()
                    && set_netmask(iface, &args[i]) != 0 { return 1; }
            }
            "mtu" => {
                i += 1;
                if i < args.len()
                    && set_mtu(iface, &args[i]) != 0 { return 1; }
            }
            addr if !addr.starts_with('-')
                && set_address(iface, addr) != 0 => { return 1; }
            _ => {}
        }
        i += 1;
    }

    0
}

fn show_all_interfaces() -> i32 {
    // Read from /proc/net/dev
    let content = match std::fs::read_to_string("/proc/net/dev") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ifconfig: {e}");
            return 1;
        }
    };

    for line in content.lines().skip(2) {
        if let Some(colon_pos) = line.find(':') {
            let iface = line[..colon_pos].trim();
            show_interface(iface);
            println!();
        }
    }
    0
}

fn show_interface(iface: &str) -> i32 {
    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if sock < 0 {
        eprintln!("ifconfig: socket: {}", io::Error::last_os_error());
        return 1;
    }

    let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
    let name_bytes = iface.as_bytes();
    let copy_len = name_bytes.len().min(libc::IFNAMSIZ - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr(),
            ifr.ifr_name.as_mut_ptr() as *mut u8,
            copy_len,
        );
    }

    // Get flags
    let flags = if unsafe { libc::ioctl(sock, libc::SIOCGIFFLAGS as _, &mut ifr) } == 0 {
        unsafe { ifr.ifr_ifru.ifru_flags }
    } else {
        0
    };

    let up = flags as u32 & libc::IFF_UP as u32 != 0;
    let running = flags as u32 & libc::IFF_RUNNING as u32 != 0;

    print!("{iface}: flags={flags}");
    let mut flag_names = Vec::new();
    if up { flag_names.push("UP"); }
    if running { flag_names.push("RUNNING"); }
    if flags as u32 & libc::IFF_BROADCAST as u32 != 0 { flag_names.push("BROADCAST"); }
    if flags as u32 & libc::IFF_MULTICAST as u32 != 0 { flag_names.push("MULTICAST"); }
    if flags as u32 & libc::IFF_LOOPBACK as u32 != 0 { flag_names.push("LOOPBACK"); }
    println!("<{}>", flag_names.join(","));

    // Get IP address
    if unsafe { libc::ioctl(sock, libc::SIOCGIFADDR as _, &mut ifr) } == 0 {
        let addr = unsafe { &*(&ifr.ifr_ifru.ifru_addr as *const _ as *const libc::sockaddr_in) };
        let ip = u32::from_be(addr.sin_addr.s_addr);
        print!("        inet {}.{}.{}.{}",
            (ip >> 24) & 0xff, (ip >> 16) & 0xff, (ip >> 8) & 0xff, ip & 0xff);

        // Get netmask
        if unsafe { libc::ioctl(sock, libc::SIOCGIFNETMASK as _, &mut ifr) } == 0 {
            let mask = unsafe { &*(&ifr.ifr_ifru.ifru_addr as *const _ as *const libc::sockaddr_in) };
            let m = u32::from_be(mask.sin_addr.s_addr);
            print!("  netmask {}.{}.{}.{}",
                (m >> 24) & 0xff, (m >> 16) & 0xff, (m >> 8) & 0xff, m & 0xff);
        }
        println!();
    }

    // Get MTU
    if unsafe { libc::ioctl(sock, libc::SIOCGIFMTU as _, &mut ifr) } == 0 {
        let mtu = unsafe { ifr.ifr_ifru.ifru_mtu };
        println!("        mtu {mtu}");
    }

    // Get HW address
    if unsafe { libc::ioctl(sock, libc::SIOCGIFHWADDR as _, &mut ifr) } == 0 {
        let hw = unsafe { ifr.ifr_ifru.ifru_hwaddr.sa_data };
        println!("        ether {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            hw[0] as u8, hw[1] as u8, hw[2] as u8,
            hw[3] as u8, hw[4] as u8, hw[5] as u8);
    }

    unsafe { libc::close(sock); }
    0
}

fn set_address(iface: &str, addr: &str) -> i32 {
    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if sock < 0 { return 1; }

    let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
    let name_bytes = iface.as_bytes();
    let copy_len = name_bytes.len().min(libc::IFNAMSIZ - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr(),
            ifr.ifr_name.as_mut_ptr() as *mut u8,
            copy_len,
        );
    }

    let c_addr = CString::new(addr).unwrap();
    let sin = unsafe { &mut *(&mut ifr.ifr_ifru.ifru_addr as *mut _ as *mut libc::sockaddr_in) };
    sin.sin_family = libc::AF_INET as u16;
    unsafe { inet_pton(libc::AF_INET, c_addr.as_ptr(), &mut sin.sin_addr as *mut _ as *mut _) };

    let ret = unsafe { libc::ioctl(sock, libc::SIOCSIFADDR as _, &ifr) };
    unsafe { libc::close(sock); }

    if ret < 0 {
        eprintln!("ifconfig: SIOCSIFADDR: {}", io::Error::last_os_error());
        return 1;
    }
    0
}

fn set_netmask(iface: &str, mask: &str) -> i32 {
    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if sock < 0 { return 1; }

    let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
    let name_bytes = iface.as_bytes();
    let copy_len = name_bytes.len().min(libc::IFNAMSIZ - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr(),
            ifr.ifr_name.as_mut_ptr() as *mut u8,
            copy_len,
        );
    }

    let c_mask = CString::new(mask).unwrap();
    let sin = unsafe { &mut *(&mut ifr.ifr_ifru.ifru_addr as *mut _ as *mut libc::sockaddr_in) };
    sin.sin_family = libc::AF_INET as u16;
    unsafe { inet_pton(libc::AF_INET, c_mask.as_ptr(), &mut sin.sin_addr as *mut _ as *mut _) };

    let ret = unsafe { libc::ioctl(sock, libc::SIOCSIFNETMASK as _, &ifr) };
    unsafe { libc::close(sock); }

    if ret < 0 {
        eprintln!("ifconfig: SIOCSIFNETMASK: {}", io::Error::last_os_error());
        return 1;
    }
    0
}

fn set_mtu(iface: &str, mtu_str: &str) -> i32 {
    let mtu: i32 = match mtu_str.parse() {
        Ok(m) => m,
        Err(_) => { eprintln!("ifconfig: invalid MTU: {mtu_str}"); return 1; }
    };

    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if sock < 0 { return 1; }

    let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
    let name_bytes = iface.as_bytes();
    let copy_len = name_bytes.len().min(libc::IFNAMSIZ - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr(),
            ifr.ifr_name.as_mut_ptr() as *mut u8,
            copy_len,
        );
        ifr.ifr_ifru.ifru_mtu = mtu;
    }

    let ret = unsafe { libc::ioctl(sock, libc::SIOCSIFMTU as _, &ifr) };
    unsafe { libc::close(sock); }

    if ret < 0 {
        eprintln!("ifconfig: SIOCSIFMTU: {}", io::Error::last_os_error());
        return 1;
    }
    0
}

fn set_interface_flags(iface: &str, up: bool) -> i32 {
    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if sock < 0 { return 1; }

    let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
    let name_bytes = iface.as_bytes();
    let copy_len = name_bytes.len().min(libc::IFNAMSIZ - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr(),
            ifr.ifr_name.as_mut_ptr() as *mut u8,
            copy_len,
        );
    }

    // Get current flags
    if unsafe { libc::ioctl(sock, libc::SIOCGIFFLAGS as _, &mut ifr) } < 0 {
        unsafe { libc::close(sock); }
        return 1;
    }

    unsafe {
        if up {
            ifr.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as i16;
        } else {
            ifr.ifr_ifru.ifru_flags &= !(libc::IFF_UP as i16);
        }
    }

    let ret = unsafe { libc::ioctl(sock, libc::SIOCSIFFLAGS as _, &ifr) };
    unsafe { libc::close(sock); }

    if ret < 0 {
        eprintln!("ifconfig: SIOCSIFFLAGS: {}", io::Error::last_os_error());
        return 1;
    }
    0
}
