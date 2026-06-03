/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ipcs — show information on IPC facilities

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut show_shm = false;
    let mut show_msg = false;
    let mut show_sem = false;

    for arg in args {
        match arg.as_str() {
            "-m" => show_shm = true,
            "-q" => show_msg = true,
            "-s" => show_sem = true,
            "-a" | "--all" => { show_shm = true; show_msg = true; show_sem = true; }
            "-h" | "--help" => {
                eprintln!("Usage: ipcs [-amqs]");
                return 0;
            }
            _ => {}
        }
    }

    // If nothing specified, show all
    if !show_shm && !show_msg && !show_sem {
        show_shm = true;
        show_msg = true;
        show_sem = true;
    }

    if show_shm {
        println!("\n------ Shared Memory Segments --------");
        println!("{:<10} {:<10} {:<10} {:<10} {:<10}", "key", "shmid", "owner", "perms", "bytes");
        if let Ok(content) = fs::read_to_string("/proc/sysvipc/shm") {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    println!("{:<10} {:<10} {:<10} {:<10} {:<10}",
                        parts.first().unwrap_or(&""),
                        parts.get(1).unwrap_or(&""),
                        parts.get(2).unwrap_or(&""),
                        parts.get(3).unwrap_or(&""),
                        parts.get(4).unwrap_or(&""));
                }
            }
        }
    }

    if show_msg {
        println!("\n------ Message Queues --------");
        println!("{:<10} {:<10} {:<10} {:<10} {:<10}", "key", "msqid", "owner", "perms", "used-bytes");
        if let Ok(content) = fs::read_to_string("/proc/sysvipc/msg") {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    println!("{:<10} {:<10} {:<10} {:<10} {:<10}",
                        parts.first().unwrap_or(&""),
                        parts.get(1).unwrap_or(&""),
                        parts.get(2).unwrap_or(&""),
                        parts.get(3).unwrap_or(&""),
                        parts.get(4).unwrap_or(&""));
                }
            }
        }
    }

    if show_sem {
        println!("\n------ Semaphore Arrays --------");
        println!("{:<10} {:<10} {:<10} {:<10} {:<10}", "key", "semid", "owner", "perms", "nsems");
        if let Ok(content) = fs::read_to_string("/proc/sysvipc/sem") {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    println!("{:<10} {:<10} {:<10} {:<10} {:<10}",
                        parts.first().unwrap_or(&""),
                        parts.get(1).unwrap_or(&""),
                        parts.get(2).unwrap_or(&""),
                        parts.get(3).unwrap_or(&""),
                        parts.get(4).unwrap_or(&""));
                }
            }
        }
    }

    0
}
