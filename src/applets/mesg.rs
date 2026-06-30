/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! mesg — control write access to your terminal
use std::os::unix::fs::PermissionsExt;

pub fn run(args: &[String]) -> i32 {
    let tty = match current_tty() {
        Some(t) => t,
        None => { eprintln!("mesg: not a tty"); return 1; }
    };

    let meta = match std::fs::metadata(&tty) {
        Ok(m) => m,
        Err(e) => { eprintln!("mesg: {tty}: {e}"); return 1; }
    };
    let mode = meta.permissions().mode();

    match args.first().map(|s| s.as_str()) {
        None => {
            // Report current state: group write bit (0o020).
            if mode & 0o020 != 0 {
                println!("is y");
            } else {
                println!("is n");
            }
            0
        }
        Some("y") => set_mode(&tty, mode | 0o020),
        Some("n") => set_mode(&tty, mode & !0o020),
        Some(other) => { eprintln!("mesg: invalid argument: {other}"); 1 }
    }
}

fn set_mode(tty: &str, mode: u32) -> i32 {
    let perm = std::fs::Permissions::from_mode(mode);
    if let Err(e) = std::fs::set_permissions(tty, perm) {
        eprintln!("mesg: {tty}: {e}");
        return 1;
    }
    0
}

fn current_tty() -> Option<String> {
    for fd in 0..3 {
        if unsafe { libc::isatty(fd) } == 1 {
            let p = std::fs::read_link(format!("/proc/self/fd/{fd}")).ok()?;
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}
