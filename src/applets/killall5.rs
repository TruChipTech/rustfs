/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! killall5 — send a signal to all processes (SysV style)
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut signal = libc::SIGTERM;
    let mut omit: Vec<i32> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "-o" {
            i += 1;
            if let Some(v) = args.get(i) { if let Ok(p) = v.parse() { omit.push(p); } }
        } else if let Some(sig) = a.strip_prefix('-') {
            if let Ok(n) = sig.parse::<i32>() { signal = n; }
        }
        i += 1;
    }

    let me = std::process::id() as i32;
    let parent = unsafe { libc::getppid() };
    let session = unsafe { libc::getsid(0) };

    // Stop scheduling while we scan, like the real killall5.
    unsafe { libc::kill(-1, libc::SIGSTOP); }

    let dir = match fs::read_dir("/proc") { Ok(d) => d, Err(_) => { unsafe { libc::kill(-1, libc::SIGCONT); } return 1; } };
    for entry in dir.flatten() {
        let dname = entry.file_name().to_string_lossy().to_string();
        let pid: i32 = match dname.parse() { Ok(p) => p, Err(_) => continue };
        if pid <= 1 || pid == me || pid == parent || omit.contains(&pid) {
            continue;
        }
        // Skip processes in our own session (the controlling shell chain).
        if let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) {
            if let Some(sid) = stat.rsplit(')').next().and_then(|s| s.split_whitespace().nth(3)) {
                if sid.parse::<i32>().ok() == Some(session) { continue; }
            }
        }
        unsafe { libc::kill(pid, signal); }
    }

    unsafe { libc::kill(-1, libc::SIGCONT); }
    0
}
