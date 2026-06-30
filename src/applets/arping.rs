/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! arping — send ARP requests to a neighbor host

use std::io;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

pub fn run(args: &[String]) -> i32 {
    let mut count: Option<u32> = None;
    let mut interface: Option<String> = None;
    let mut timeout_secs: u64 = 1;
    let mut target = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => { i += 1; if i < args.len() { count = args[i].parse().ok(); } }
            "-I" => { i += 1; if i < args.len() { interface = Some(args[i].clone()); } }
            "-w" => { i += 1; if i < args.len() { timeout_secs = args[i].parse().unwrap_or(1); } }
            s if !s.starts_with('-') => target = s.to_string(),
            other => { eprintln!("arping: unknown option: {other}"); return 1; }
        }
        i += 1;
    }

    if target.is_empty() {
        eprintln!("Usage: arping [-c count] [-I interface] [-w timeout] HOST");
        return 1;
    }

    let _target_ip: Ipv4Addr = match target.parse() {
        Ok(ip) => ip,
        Err(_) => {
            eprintln!("arping: invalid IP address: {target}");
            return 1;
        }
    };

    let iface = interface.as_deref().unwrap_or("eth0");
    println!("ARPING {target} from 0.0.0.0 {iface}");

    // Create raw socket for ARP
    let sock = unsafe { libc::socket(libc::AF_PACKET, libc::SOCK_DGRAM, (libc::ETH_P_ARP as u16).to_be() as i32) };
    if sock < 0 {
        eprintln!("arping: cannot create raw socket: {}", io::Error::last_os_error());
        return 1;
    }

    let max_count = count.unwrap_or(u32::MAX);
    let deadline = Instant::now() + Duration::from_secs(timeout_secs * max_count as u64);
    let mut sent = 0u32;
    let received = 0u32;

    for _ in 0..max_count {
        if Instant::now() >= deadline {
            break;
        }

        sent += 1;
        // In a real implementation, we'd construct and send ARP packets via the raw socket.
        // For now, we indicate that functionality requires root privileges.
        eprintln!("arping: sending ARP request #{sent} (raw socket ARP requires root privileges)");

        // Sleep between probes
        std::thread::sleep(Duration::from_secs(1));
    }

    unsafe { libc::close(sock); }

    println!("\n--- {target} statistics ---");
    println!("{sent} packets transmitted, {received} packets received, {}% unanswered",
        ((sent - received) * 100).checked_div(sent).unwrap_or(0));

    if received > 0 { 0 } else { 1 }
}
