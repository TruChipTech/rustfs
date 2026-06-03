/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! fuser — identify processes using files or sockets

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut kill_signal: Option<i32> = None;
    let mut verbose = false;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-k" | "--kill" => kill_signal = Some(libc::SIGKILL),
            "-s" | "--signal" => {
                i += 1;
                if i < args.len() { kill_signal = parse_signal(&args[i]); }
            }
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => {
                eprintln!("Usage: fuser [-k] [-s SIGNAL] [-v] FILE...");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    if files.is_empty() {
        eprintln!("Usage: fuser [-k] [-s SIGNAL] [-v] FILE...");
        return 1;
    }

    let mut found_any = false;

    for file in &files {
        let pids = find_processes_using(file);
        if pids.is_empty() {
            continue;
        }

        found_any = true;
        if verbose {
            println!("{file}:");
        }

        let pid_strs: Vec<String> = pids.iter().map(|p| p.to_string()).collect();
        if verbose {
            for pid in &pids {
                let comm = get_process_name(*pid);
                println!("  {pid} {comm}");
            }
        } else {
            print!("{file}: ");
            println!("{}", pid_strs.join(" "));
        }

        if let Some(sig) = kill_signal {
            for pid in &pids {
                unsafe { libc::kill(*pid, sig); }
            }
        }
    }

    if found_any { 0 } else { 1 }
}

fn find_processes_using(file: &str) -> Vec<i32> {
    let mut pids = Vec::new();

    // Resolve to absolute path
    let target = match fs::canonicalize(file) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => file.to_string(),
    };

    // Scan /proc/*/fd/
    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return pids,
    };

    for entry in proc_dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let pid: i32 = match name.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let fd_dir = format!("/proc/{pid}/fd");
        if let Ok(fds) = fs::read_dir(&fd_dir) {
            for fd_entry in fds.flatten() {
                if let Ok(link) = fs::read_link(fd_entry.path()) {
                    if link.to_string_lossy() == target {
                        pids.push(pid);
                        break;
                    }
                }
            }
        }
    }

    pids
}

fn get_process_name(pid: i32) -> String {
    fs::read_to_string(format!("/proc/{pid}/comm"))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn parse_signal(s: &str) -> Option<i32> {
    match s.to_uppercase().trim_start_matches("SIG") {
        "KILL" | "9" => Some(9),
        "TERM" | "15" => Some(15),
        "HUP" | "1" => Some(1),
        "INT" | "2" => Some(2),
        "QUIT" | "3" => Some(3),
        _ => s.parse().ok(),
    }
}
