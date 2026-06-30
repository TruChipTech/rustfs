/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! chvt — switch the foreground virtual terminal.

use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

const VT_ACTIVATE: libc::c_ulong = 0x5606;
const VT_WAITACTIVE: libc::c_ulong = 0x5607;

pub fn run(args: &[String]) -> i32 {
    let n: Option<libc::c_ulong> = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .and_then(|s| s.parse().ok());
    let n = match n {
        Some(n) => n,
        None => {
            eprintln!("Usage: chvt N");
            return 1;
        }
    };

    let console = match OpenOptions::new().read(true).write(true).open("/dev/console") {
        Ok(f) => f,
        Err(_) => match OpenOptions::new().read(true).write(true).open("/dev/tty") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("chvt: cannot open console: {e}");
                return 1;
            }
        },
    };
    let fd = console.as_raw_fd();
    unsafe {
        if libc::ioctl(fd, VT_ACTIVATE as _, n) != 0 {
            eprintln!("chvt: VT_ACTIVATE failed: {}", std::io::Error::last_os_error());
            return 1;
        }
        if libc::ioctl(fd, VT_WAITACTIVE as _, n) != 0 {
            eprintln!("chvt: VT_WAITACTIVE failed: {}", std::io::Error::last_os_error());
            return 1;
        }
    }
    0
}
