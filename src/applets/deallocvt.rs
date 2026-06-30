/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! deallocvt — deallocate unused virtual terminals.

use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

const VT_DISALLOCATE: libc::c_ulong = 0x5608;

pub fn run(args: &[String]) -> i32 {
    // N == 0 deallocates all unused VTs (the default).
    let n: libc::c_ulong = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let console = match OpenOptions::new().read(true).write(true).open("/dev/console") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("deallocvt: cannot open console: {e}");
            return 1;
        }
    };
    unsafe {
        if libc::ioctl(console.as_raw_fd(), VT_DISALLOCATE as _, n) != 0 {
            eprintln!(
                "deallocvt: VT_DISALLOCATE failed: {}",
                std::io::Error::last_os_error()
            );
            return 1;
        }
    }
    0
}
