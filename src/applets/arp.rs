/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! arp — manipulate the ARP cache

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut show_all = false;
    let mut numeric = false;
    let mut host: Option<&str> = None;
    let mut delete = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" => show_all = true,
            "-n" => numeric = true,
            "-d" => {
                delete = true;
                i += 1;
                if i < args.len() {
                    host = Some(&args[i]);
                }
            }
            s if !s.starts_with('-') => host = Some(s),
            other => {
                eprintln!("arp: unknown option: {other}");
                return 1;
            }
        }
        i += 1;
    }

    if delete {
        if let Some(h) = host {
            return delete_arp_entry(h);
        } else {
            eprintln!("arp: -d requires a hostname");
            return 1;
        }
    }

    show_arp_table(host, numeric || show_all)
}

fn show_arp_table(filter_host: Option<&str>, _numeric: bool) -> i32 {
    let content = match fs::read_to_string("/proc/net/arp") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("arp: cannot read ARP table: {e}");
            return 1;
        }
    };

    for (i, line) in content.lines().enumerate() {
        if i == 0 {
            // Print header
            println!("{line}");
            continue;
        }

        if let Some(host) = filter_host {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if !fields.is_empty() && fields[0] == host {
                println!("{line}");
            }
        } else {
            println!("{line}");
        }
    }
    0
}

fn delete_arp_entry(host: &str) -> i32 {
    // Use SIOCDARP ioctl — for simplicity, shell out
    let status = std::process::Command::new("/sbin/arp")
        .args(["-d", host])
        .status();

    match status {
        Ok(s) if s.success() => 0,
        _ => {
            eprintln!("arp: cannot delete entry for {host}");
            1
        }
    }
}
