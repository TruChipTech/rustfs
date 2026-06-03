/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ifup — bring a network interface up

use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut interfaces: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-a" | "--all" => {
                return up_all_interfaces();
            }
            "-h" | "--help" => {
                eprintln!("Usage: ifup [-a] IFACE...");
                return 0;
            }
            s if !s.starts_with('-') => interfaces.push(s.to_string()),
            _ => {}
        }
    }

    if interfaces.is_empty() {
        eprintln!("Usage: ifup [-a] IFACE...");
        return 1;
    }

    let mut exit_code = 0;
    for iface in &interfaces {
        if bring_up(iface) != 0 {
            exit_code = 1;
        }
    }
    exit_code
}

fn bring_up(iface: &str) -> i32 {
    // Bring interface up using ioctl
    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if sock < 0 {
        eprintln!("ifup: socket error");
        return 1;
    }

    let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
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
        eprintln!("ifup: {iface}: {}", std::io::Error::last_os_error());
        unsafe { libc::close(sock); }
        return 1;
    }

    // Set UP flag
    unsafe { ifr.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as i16; }

    let ret = unsafe { libc::ioctl(sock, libc::SIOCSIFFLAGS as _, &ifr) };
    unsafe { libc::close(sock); }

    if ret < 0 {
        eprintln!("ifup: {iface}: {}", std::io::Error::last_os_error());
        return 1;
    }

    // Run /etc/network/if-up.d scripts
    let script = format!("/etc/network/if-up.d/{iface}");
    if std::path::Path::new(&script).exists() {
        let _ = Command::new(&script).status();
    }

    println!("ifup: interface {iface} is up");
    0
}

fn up_all_interfaces() -> i32 {
    let content = match std::fs::read_to_string("/proc/net/dev") {
        Ok(c) => c,
        Err(e) => { eprintln!("ifup: {e}"); return 1; }
    };

    let mut exit_code = 0;
    for line in content.lines().skip(2) {
        if let Some(colon_pos) = line.find(':') {
            let iface = line[..colon_pos].trim();
            if bring_up(iface) != 0 {
                exit_code = 1;
            }
        }
    }
    exit_code
}
