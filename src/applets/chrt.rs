/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! chrt — manipulate the real-time attributes of a process
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut policy = libc::SCHED_RR;
    let mut pid: Option<i32> = None;
    let mut idx = 0;

    while idx < args.len() {
        match args[idx].as_str() {
            "-f" | "--fifo" => policy = libc::SCHED_FIFO,
            "-r" | "--rr" => policy = libc::SCHED_RR,
            "-o" | "--other" => policy = libc::SCHED_OTHER,
            "-b" | "--batch" => policy = libc::SCHED_BATCH,
            "-i" | "--idle" => policy = libc::SCHED_IDLE,
            "-p" | "--pid" => { idx += 1; pid = args.get(idx).and_then(|s| s.parse().ok()); }
            _ => break,
        }
        idx += 1;
    }

    // Query mode: chrt -p PID
    if let (Some(p), true) = (pid, idx >= args.len()) {
        let pol = unsafe { libc::sched_getscheduler(p) };
        let mut param: libc::sched_param = unsafe { std::mem::zeroed() };
        unsafe { libc::sched_getparam(p, &mut param); }
        let name = match pol {
            x if x == libc::SCHED_FIFO => "SCHED_FIFO",
            x if x == libc::SCHED_RR => "SCHED_RR",
            x if x == libc::SCHED_BATCH => "SCHED_BATCH",
            x if x == libc::SCHED_IDLE => "SCHED_IDLE",
            _ => "SCHED_OTHER",
        };
        println!("pid {p}'s current scheduling policy: {name}");
        println!("pid {p}'s current scheduling priority: {}", param.sched_priority);
        return 0;
    }

    if idx >= args.len() {
        eprintln!("Usage: chrt [-f|-r|-o|-b|-i] PRIORITY {{COMMAND|-p PID}}");
        return 1;
    }

    let prio: i32 = args[idx].parse().unwrap_or(0);
    idx += 1;
    let mut param: libc::sched_param = unsafe { std::mem::zeroed() };
    param.sched_priority = prio;

    if let Some(p) = pid {
        if unsafe { libc::sched_setscheduler(p, policy, &param) } < 0 {
            eprintln!("chrt: {}", std::io::Error::last_os_error());
            return 1;
        }
        return 0;
    }

    if unsafe { libc::sched_setscheduler(0, policy, &param) } < 0 {
        eprintln!("chrt: {}", std::io::Error::last_os_error());
        return 1;
    }
    if idx >= args.len() {
        eprintln!("chrt: missing command");
        return 1;
    }
    let prog = &args[idx];
    let err = Command::new(prog).args(&args[idx + 1..]).exec();
    eprintln!("chrt: {prog}: {err}");
    127
}
