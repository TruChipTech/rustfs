/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! nice — run a program with modified scheduling priority
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut adjust: i32 = 10;
    let mut idx = 0;
    let mut explicit = false;

    while idx < args.len() {
        let a = &args[idx];
        if a == "-n" {
            idx += 1;
            adjust = args.get(idx).and_then(|s| s.parse().ok()).unwrap_or(10);
            explicit = true;
        } else if let Some(n) = a.strip_prefix("-n") {
            adjust = n.parse().unwrap_or(10);
            explicit = true;
        } else if a.starts_with('-') && a.len() > 1 && a[1..].chars().all(|c| c.is_ascii_digit() || c == '-') {
            adjust = a[1..].parse().unwrap_or(10);
            explicit = true;
        } else {
            break;
        }
        idx += 1;
    }

    if idx >= args.len() {
        // No command: print current niceness.
        let cur = unsafe { libc::getpriority(libc::PRIO_PROCESS, 0) };
        println!("{cur}");
        return 0;
    }

    let _ = explicit;
    let cur = unsafe { libc::getpriority(libc::PRIO_PROCESS, 0) };
    unsafe { libc::setpriority(libc::PRIO_PROCESS, 0, cur + adjust); }

    let prog = &args[idx];
    let err = Command::new(prog).args(&args[idx + 1..]).exec();
    eprintln!("nice: {prog}: {err}");
    127
}
