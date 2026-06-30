/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! freeramdisk — free all memory used by the specified ramdisk
use std::os::unix::io::AsRawFd;

const BLKFLSBUF: libc::c_ulong = 0x1261;

pub fn run(args: &[String]) -> i32 {
    let dev = match args.iter().find(|a| !a.starts_with('-')) {
        Some(d) => d.clone(),
        None => { eprintln!("Usage: freeramdisk DEVICE"); return 1; }
    };

    let file = match std::fs::OpenOptions::new().read(true).write(true).open(&dev) {
        Ok(f) => f,
        Err(e) => { eprintln!("freeramdisk: {dev}: {e}"); return 1; }
    };
    if unsafe { libc::ioctl(file.as_raw_fd(), BLKFLSBUF as _) } != 0 {
        eprintln!("freeramdisk: {dev}: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}
