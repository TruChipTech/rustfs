/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ipcalc — calculate IP network settings

pub fn run(args: &[String]) -> i32 {
    let mut show_broadcast = false;
    let mut show_network = false;
    let mut show_prefix = false;
    let mut address = String::new();
    let mut netmask: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-b" | "--broadcast" => show_broadcast = true,
            "-n" | "--network" => show_network = true,
            "-p" | "--prefix" => show_prefix = true,
            "-m" | "--netmask" => {
                i += 1;
                if i < args.len() { netmask = Some(args[i].clone()); }
            }
            "-h" | "--help" => {
                eprintln!("Usage: ipcalc [-bnp] [-m NETMASK] ADDRESS[/PREFIX]");
                return 0;
            }
            s if !s.starts_with('-') => address = s.to_string(),
            _ => {}
        }
        i += 1;
    }

    if address.is_empty() {
        eprintln!("Usage: ipcalc [-bnp] ADDRESS[/PREFIX]");
        return 1;
    }

    // Parse address/prefix
    let (ip_str, prefix_len) = if let Some(slash) = address.find('/') {
        let prefix: u32 = address[slash + 1..].parse().unwrap_or(24);
        (&address[..slash], prefix)
    } else if let Some(ref mask) = netmask {
        let prefix = netmask_to_prefix(mask);
        (address.as_str(), prefix)
    } else {
        (address.as_str(), 24)
    };

    let ip = match parse_ipv4(ip_str) {
        Some(ip) => ip,
        None => {
            eprintln!("ipcalc: invalid address: {ip_str}");
            return 1;
        }
    };

    let mask = prefix_to_mask(prefix_len);
    let network = ip & mask;
    let broadcast = network | !mask;

    let show_all = !show_broadcast && !show_network && !show_prefix;

    if show_all || show_network {
        println!("NETWORK={}", format_ipv4(network));
    }
    if show_all {
        println!("NETMASK={}", format_ipv4(mask));
    }
    if show_all || show_broadcast {
        println!("BROADCAST={}", format_ipv4(broadcast));
    }
    if show_all || show_prefix {
        println!("PREFIX={prefix_len}");
    }
    if show_all {
        let hosts = if prefix_len >= 31 { 0 } else { (1u32 << (32 - prefix_len)) - 2 };
        println!("HOSTS={hosts}");
        if hosts > 0 {
            println!("MINADDR={}", format_ipv4(network + 1));
            println!("MAXADDR={}", format_ipv4(broadcast - 1));
        }
    }

    0
}

fn parse_ipv4(s: &str) -> Option<u32> {
    let parts: Vec<u8> = s.split('.').filter_map(|p| p.parse().ok()).collect();
    if parts.len() != 4 { return None; }
    Some(((parts[0] as u32) << 24) | ((parts[1] as u32) << 16) | ((parts[2] as u32) << 8) | parts[3] as u32)
}

fn format_ipv4(ip: u32) -> String {
    format!("{}.{}.{}.{}", (ip >> 24) & 0xff, (ip >> 16) & 0xff, (ip >> 8) & 0xff, ip & 0xff)
}

fn prefix_to_mask(prefix: u32) -> u32 {
    if prefix == 0 { 0 } else { !0u32 << (32 - prefix) }
}

fn netmask_to_prefix(mask: &str) -> u32 {
    if let Some(val) = parse_ipv4(mask) {
        val.count_ones()
    } else {
        24
    }
}
