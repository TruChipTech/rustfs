/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! killall — kill processes by name

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut signal = libc::SIGTERM;
    let mut names: Vec<String> = Vec::new();
    let mut quiet = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-q" | "--quiet" => quiet = true,
            "-s" | "--signal" => {
                i += 1;
                if i < args.len() { signal = parse_signal(&args[i]); }
            }
            "-l" | "--list" => {
                print_signals();
                return 0;
            }
            "-h" | "--help" => {
                eprintln!("Usage: killall [-q] [-s SIGNAL] NAME...");
                return 0;
            }
            s if s.starts_with('-') && s.len() > 1 => {
                let sig_str = &s[1..];
                let parsed = parse_signal(sig_str);
                if parsed != 0 {
                    signal = parsed;
                } else {
                    eprintln!("killall: unknown option: {s}");
                    return 1;
                }
            }
            s => names.push(s.to_string()),
        }
        i += 1;
    }

    if names.is_empty() {
        eprintln!("Usage: killall [-q] [-s SIGNAL] NAME...");
        return 1;
    }

    let mut exit_code = 0;
    for name in &names {
        let pids = find_pids_by_name(name);
        if pids.is_empty() {
            if !quiet {
                eprintln!("killall: {name}: no process found");
            }
            exit_code = 1;
            continue;
        }
        for pid in &pids {
            if unsafe { libc::kill(*pid, signal) } != 0 {
                if !quiet {
                    eprintln!("killall: kill({pid}): {}", std::io::Error::last_os_error());
                }
                exit_code = 1;
            }
        }
    }
    exit_code
}

fn find_pids_by_name(name: &str) -> Vec<i32> {
    let mut pids = Vec::new();
    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return pids,
    };

    for entry in proc_dir.flatten() {
        let dir_name = entry.file_name().to_string_lossy().to_string();
        if !dir_name.chars().all(|c| c.is_ascii_digit()) { continue; }
        let pid: i32 = match dir_name.parse() { Ok(p) => p, Err(_) => continue };

        let comm_path = format!("/proc/{pid}/comm");
        if let Ok(comm) = fs::read_to_string(&comm_path) {
            if comm.trim() == name {
                pids.push(pid);
            }
        }
    }
    pids
}

fn parse_signal(s: &str) -> i32 {
    if let Ok(n) = s.parse::<i32>() { return n; }
    let upper = s.to_uppercase();
    let name = upper.strip_prefix("SIG").unwrap_or(&upper);
    match name {
        "HUP" => 1, "INT" => 2, "QUIT" => 3, "KILL" => 9,
        "TERM" => 15, "STOP" => 19, "CONT" => 18, "USR1" => 10, "USR2" => 12,
        _ => 0,
    }
}

fn print_signals() {
    let signals = ["HUP", "INT", "QUIT", "ILL", "TRAP", "ABRT", "BUS", "FPE",
        "KILL", "USR1", "SEGV", "USR2", "PIPE", "ALRM", "TERM"];
    for (i, sig) in signals.iter().enumerate() {
        print!("{:2}) SIG{sig:<8}", i + 1);
        if (i + 1) % 5 == 0 { println!(); }
    }
    println!();
}
