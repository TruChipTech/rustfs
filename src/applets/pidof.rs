/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! pidof — find the process ID of a running program
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut single = false;
    let mut omit: Vec<i32> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" => single = true,
            "-o" => { i += 1; if let Some(v) = args.get(i) { omit.extend(v.split(',').filter_map(|s| s.parse::<i32>().ok())); } }
            "-x" => {} // also match scripts — treated same here
            s => names.push(s.to_string()),
        }
        i += 1;
    }
    if names.is_empty() {
        return 1;
    }

    let mut pids: Vec<i32> = Vec::new();
    for name in &names {
        let base = name.rsplit('/').next().unwrap_or(name);
        for pid in matching_pids(base) {
            if !omit.contains(&pid) && !pids.contains(&pid) {
                pids.push(pid);
                if single { break; }
            }
        }
        if single && !pids.is_empty() { break; }
    }

    if pids.is_empty() {
        return 1;
    }
    let strs: Vec<String> = pids.iter().map(|p| p.to_string()).collect();
    println!("{}", strs.join(" "));
    0
}

fn matching_pids(base: &str) -> Vec<i32> {
    let mut pids = Vec::new();
    let dir = match fs::read_dir("/proc") { Ok(d) => d, Err(_) => return pids };
    for entry in dir.flatten() {
        let dname = entry.file_name().to_string_lossy().to_string();
        let pid: i32 = match dname.parse() { Ok(p) => p, Err(_) => continue };
        if let Ok(comm) = fs::read_to_string(format!("/proc/{pid}/comm")) {
            if comm.trim() == base {
                pids.push(pid);
                continue;
            }
        }
        if let Ok(cmd) = fs::read_to_string(format!("/proc/{pid}/cmdline")) {
            if let Some(arg0) = cmd.split('\0').next() {
                let b = arg0.rsplit('/').next().unwrap_or(arg0);
                if b == base { pids.push(pid); }
            }
        }
    }
    pids.sort_unstable_by(|a, b| b.cmp(a));
    pids
}
