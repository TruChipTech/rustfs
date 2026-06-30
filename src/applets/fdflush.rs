/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! fdflush — force the kernel to drop its cached view of a floppy disk
use std::os::unix::io::AsRawFd;

const FDFLUSH: libc::c_ulong = 0x024b;

pub fn run(args: &[String]) -> i32 {
    let dev = match args.iter().find(|a| !a.starts_with('-')) {
        Some(d) => d.clone(),
        None => { eprintln!("Usage: fdflush DEVICE"); return 1; }
    };
    let file = match std::fs::File::open(&dev) {
        Ok(f) => f,
        Err(e) => { eprintln!("fdflush: {dev}: {e}"); return 1; }
    };
    if unsafe { libc::ioctl(file.as_raw_fd(), FDFLUSH as _) } != 0 {
        eprintln!("fdflush: {dev}: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}
