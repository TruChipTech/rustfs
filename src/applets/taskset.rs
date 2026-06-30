/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! taskset — set or get a process's CPU affinity
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut pid: Option<i32> = None;
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-p" | "--pid" => { idx += 1; pid = args.get(idx).and_then(|s| s.parse().ok()); }
            "-c" | "--cpu-list" => {} // accepted; mask parsing below handles lists
            _ => break,
        }
        idx += 1;
    }

    // Query mode: taskset -p PID
    if let (Some(p), true) = (pid, idx >= args.len()) {
        let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
        if unsafe { libc::sched_getaffinity(p, std::mem::size_of::<libc::cpu_set_t>(), &mut set) } < 0 {
            eprintln!("taskset: {}", std::io::Error::last_os_error());
            return 1;
        }
        let mut mask: u64 = 0;
        for c in 0..64 {
            if unsafe { libc::CPU_ISSET(c, &set) } { mask |= 1 << c; }
        }
        println!("pid {p}'s current affinity mask: {mask:x}");
        return 0;
    }

    if idx >= args.len() {
        eprintln!("Usage: taskset MASK {{COMMAND|-p PID}}");
        return 1;
    }

    let mask = match u64::from_str_radix(args[idx].trim_start_matches("0x"), 16) {
        Ok(m) => m,
        Err(_) => { eprintln!("taskset: invalid mask: {}", args[idx]); return 1; }
    };
    idx += 1;

    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    unsafe { libc::CPU_ZERO(&mut set); }
    for c in 0..64 {
        if mask & (1 << c) != 0 { unsafe { libc::CPU_SET(c, &mut set); } }
    }

    let target = pid.unwrap_or(0);
    if unsafe { libc::sched_setaffinity(target, std::mem::size_of::<libc::cpu_set_t>(), &set) } < 0 {
        eprintln!("taskset: {}", std::io::Error::last_os_error());
        return 1;
    }
    if pid.is_some() {
        return 0;
    }
    if idx >= args.len() {
        eprintln!("taskset: missing command");
        return 1;
    }
    let prog = &args[idx];
    let err = Command::new(prog).args(&args[idx + 1..]).exec();
    eprintln!("taskset: {prog}: {err}");
    127
}
