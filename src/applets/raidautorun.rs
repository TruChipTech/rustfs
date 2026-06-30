/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! raidautorun — tell the kernel to automatically search and start RAID arrays
use std::os::unix::io::AsRawFd;

const RAID_AUTORUN: libc::c_ulong = 0x0936;

pub fn run(args: &[String]) -> i32 {
    let dev = args.iter().find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| "/dev/md0".to_string());

    let file = match std::fs::File::open(&dev) {
        Ok(f) => f,
        Err(e) => { eprintln!("raidautorun: {dev}: {e}"); return 1; }
    };
    if unsafe { libc::ioctl(file.as_raw_fd(), RAID_AUTORUN as _, 0) } != 0 {
        eprintln!("raidautorun: {dev}: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}
