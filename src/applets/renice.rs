/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! renice — alter priority of running processes
pub fn run(args: &[String]) -> i32 {
    let mut which = libc::PRIO_PROCESS;
    let mut prio: Option<i32> = None;
    let mut relative = false;
    let mut exit_code = 0;

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "-n" => { i += 1; prio = args.get(i).and_then(|s| s.parse().ok()); relative = true; }
            "-g" => which = libc::PRIO_PGRP,
            "-u" => which = libc::PRIO_USER,
            "-p" => which = libc::PRIO_PROCESS,
            s => {
                if prio.is_none() {
                    prio = s.parse().ok();
                } else if let (Ok(id), Some(p)) = (s.parse::<u32>(), prio) {
                    let target = if relative {
                        let cur = unsafe { libc::getpriority(which, id) };
                        cur + p
                    } else { p };
                    if unsafe { libc::setpriority(which, id, target) } != 0 {
                        eprintln!("renice: {id}: {}", std::io::Error::last_os_error());
                        exit_code = 1;
                    }
                }
            }
        }
        i += 1;
    }
    if prio.is_none() {
        eprintln!("Usage: renice [-n] PRIORITY [-p|-g|-u] ID...");
        return 1;
    }
    exit_code
}
