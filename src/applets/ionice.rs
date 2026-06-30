/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ionice — get/set program I/O scheduling class and priority
use std::os::unix::process::CommandExt;
use std::process::Command;

const IOPRIO_WHO_PROCESS: libc::c_int = 1;
const IOPRIO_CLASS_SHIFT: u32 = 13;

pub fn run(args: &[String]) -> i32 {
    let mut class: Option<i32> = None;
    let mut data: i32 = 4;
    let mut pid: Option<i32> = None;
    let mut idx = 0;

    while idx < args.len() {
        let a = &args[idx];
        match a.as_str() {
            "-c" => { idx += 1; class = args.get(idx).and_then(|s| s.parse().ok()); }
            "-n" => { idx += 1; data = args.get(idx).and_then(|s| s.parse().ok()).unwrap_or(4); }
            "-p" => { idx += 1; pid = args.get(idx).and_then(|s| s.parse().ok()); }
            _ => break,
        }
        idx += 1;
    }

    // Query mode: -p with no class and no command.
    if class.is_none() && idx >= args.len() {
        let target = pid.unwrap_or(0);
        let res = unsafe { libc::syscall(libc::SYS_ioprio_get, IOPRIO_WHO_PROCESS, target) };
        if res < 0 {
            eprintln!("ionice: {}", std::io::Error::last_os_error());
            return 1;
        }
        let cls = (res >> IOPRIO_CLASS_SHIFT) & 0x7;
        let prio = res & 0xff;
        let names = ["none", "realtime", "best-effort", "idle"];
        println!("{}: prio {}", names.get(cls as usize).unwrap_or(&"unknown"), prio);
        return 0;
    }

    let cls = class.unwrap_or(2);
    let ioprio = (cls << IOPRIO_CLASS_SHIFT) | data;

    if let Some(p) = pid {
        if unsafe { libc::syscall(libc::SYS_ioprio_set, IOPRIO_WHO_PROCESS, p, ioprio) } < 0 {
            eprintln!("ionice: {}", std::io::Error::last_os_error());
            return 1;
        }
        return 0;
    }

    if idx >= args.len() {
        eprintln!("Usage: ionice [-c CLASS] [-n PRIO] [-p PID] [COMMAND]");
        return 1;
    }
    unsafe { libc::syscall(libc::SYS_ioprio_set, IOPRIO_WHO_PROCESS, 0, ioprio); }
    let prog = &args[idx];
    let err = Command::new(prog).args(&args[idx + 1..]).exec();
    eprintln!("ionice: {prog}: {err}");
    127
}
