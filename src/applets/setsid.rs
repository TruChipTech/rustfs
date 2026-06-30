/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! setsid — run a program in a new session
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut idx = 0;
    let mut ctty = false;
    while idx < args.len() && args[idx].starts_with('-') {
        if args[idx] == "-c" { ctty = true; }
        idx += 1;
    }
    if idx >= args.len() {
        eprintln!("Usage: setsid [-c] program [args]");
        return 1;
    }

    let prog = &args[idx];
    let rest = &args[idx + 1..];

    let mut cmd = Command::new(prog);
    cmd.args(rest);
    unsafe {
        cmd.pre_exec(move || {
            if libc::setsid() < 0 {
                // Already a session leader: fork so the child can become one.
                let pid = libc::fork();
                if pid < 0 { return Err(std::io::Error::last_os_error()); }
                if pid > 0 { libc::_exit(0); }
                libc::setsid();
            }
            if ctty {
                libc::ioctl(0, libc::TIOCSCTTY, 0);
            }
            Ok(())
        });
    }
    let err = cmd.exec();
    eprintln!("setsid: {prog}: {err}");
    127
}
