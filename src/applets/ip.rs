/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ip — show / manipulate routing, network devices, interfaces and tunnels

use std::fs;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: ip [OPTIONS] OBJECT {{ COMMAND }}");
        eprintln!("OBJECT := {{ link | addr | route | neigh }}");
        return 1;
    }

    let object = &args[0];
    let sub_args = &args[1..];

    match object.as_str() {
        "link" => ip_link(sub_args),
        "addr" | "address" => ip_addr(sub_args),
        "route" | "r" => ip_route(sub_args),
        "neigh" | "neighbor" => ip_neigh(sub_args),
        "-h" | "--help" | "help" => {
            println!("Usage: ip [OPTIONS] OBJECT {{ COMMAND }}");
            println!("OBJECT := {{ link | addr | route | neigh }}");
            0
        }
        _ => {
            eprintln!("ip: unknown object '{object}'");
            1
        }
    }
}

fn ip_link(args: &[String]) -> i32 {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("show");
    match cmd {
        "show" | "list" | "ls" => {
            let content = match fs::read_to_string("/proc/net/dev") {
                Ok(c) => c,
                Err(e) => { eprintln!("ip: {e}"); return 1; }
            };
            let mut idx = 1;
            for line in content.lines().skip(2) {
                if let Some(colon_pos) = line.find(':') {
                    let iface = line[..colon_pos].trim();
                    println!("{idx}: {iface}: <UP,BROADCAST,MULTICAST>");
                    idx += 1;
                }
            }
            0
        }
        "set" => {
            if args.len() < 3 {
                eprintln!("Usage: ip link set DEV {{ up | down }}");
                return 1;
            }
            let dev = &args[1];
            let action = &args[2];
            let up = action == "up";
            set_link_state(dev, up)
        }
        _ => { eprintln!("ip link: unknown command '{cmd}'"); 1 }
    }
}

fn ip_addr(args: &[String]) -> i32 {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("show");
    match cmd {
        "show" | "list" | "ls" => {
            // Read addresses from /proc/net/if_inet6 and /proc/net/fib_trie
            let content = match fs::read_to_string("/proc/net/dev") {
                Ok(c) => c,
                Err(e) => { eprintln!("ip: {e}"); return 1; }
            };
            let mut idx = 1;
            for line in content.lines().skip(2) {
                if let Some(colon_pos) = line.find(':') {
                    let iface = line[..colon_pos].trim();
                    println!("{idx}: {iface}");
                    // Try to get address via sysfs
                    let operstate = fs::read_to_string(format!("/sys/class/net/{iface}/operstate"))
                        .unwrap_or_else(|_| "unknown".to_string());
                    println!("    state {}", operstate.trim());
                    idx += 1;
                }
            }
            0
        }
        "add" => {
            if args.len() < 4 {
                eprintln!("Usage: ip addr add ADDRESS/PREFIX dev DEV");
                return 1;
            }
            eprintln!("ip addr add: requires netlink support (use ifconfig for basic config)");
            1
        }
        "del" => {
            eprintln!("ip addr del: requires netlink support");
            1
        }
        _ => { eprintln!("ip addr: unknown command '{cmd}'"); 1 }
    }
}

fn ip_route(args: &[String]) -> i32 {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("show");
    match cmd {
        "show" | "list" | "ls" => {
            let content = match fs::read_to_string("/proc/net/route") {
                Ok(c) => c,
                Err(e) => { eprintln!("ip: {e}"); return 1; }
            };
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 8 {
                    let iface = parts[0];
                    let dest = parse_hex_ip(parts[1]);
                    let gateway = parse_hex_ip(parts[2]);
                    let mask = parse_hex_ip(parts[7]);
                    if dest == "0.0.0.0" {
                        println!("default via {gateway} dev {iface}");
                    } else {
                        println!("{dest}/{} dev {iface}", mask_to_prefix(&mask));
                    }
                }
            }
            0
        }
        _ => { eprintln!("ip route: unknown command '{cmd}'"); 1 }
    }
}

fn ip_neigh(args: &[String]) -> i32 {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("show");
    match cmd {
        "show" | "list" | "ls" => {
            let content = match fs::read_to_string("/proc/net/arp") {
                Ok(c) => c,
                Err(e) => { eprintln!("ip: {e}"); return 1; }
            };
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    println!("{} dev {} lladdr {} {}", parts[0], parts[5], parts[3],
                        if parts[2] == "0x2" { "REACHABLE" } else { "STALE" });
                }
            }
            0
        }
        _ => { eprintln!("ip neigh: unknown command '{cmd}'"); 1 }
    }
}

fn set_link_state(dev: &str, up: bool) -> i32 {
    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if sock < 0 { return 1; }

    let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
    let name_bytes = dev.as_bytes();
    let copy_len = name_bytes.len().min(libc::IFNAMSIZ - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), ifr.ifr_name.as_mut_ptr() as *mut u8, copy_len);
        libc::ioctl(sock, libc::SIOCGIFFLAGS as _, &mut ifr);
        if up {
            ifr.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as i16;
        } else {
            ifr.ifr_ifru.ifru_flags &= !(libc::IFF_UP as i16);
        }
        libc::ioctl(sock, libc::SIOCSIFFLAGS as _, &ifr);
        libc::close(sock);
    }
    0
}

fn parse_hex_ip(hex: &str) -> String {
    if let Ok(val) = u32::from_str_radix(hex.trim(), 16) {
        format!("{}.{}.{}.{}", val & 0xff, (val >> 8) & 0xff, (val >> 16) & 0xff, (val >> 24) & 0xff)
    } else {
        "0.0.0.0".to_string()
    }
}

fn mask_to_prefix(mask: &str) -> u32 {
    let parts: Vec<u8> = mask.split('.').filter_map(|s| s.parse().ok()).collect();
    if parts.len() != 4 { return 0; }
    let val = ((parts[0] as u32) << 24) | ((parts[1] as u32) << 16) | ((parts[2] as u32) << 8) | parts[3] as u32;
    val.count_ones()
}
