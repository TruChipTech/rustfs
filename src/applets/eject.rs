/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! eject — eject removable media
use std::os::unix::io::AsRawFd;

const CDROMEJECT: libc::c_ulong = 0x5309;
const CDROMCLOSETRAY: libc::c_ulong = 0x5319;

pub fn run(args: &[String]) -> i32 {
    let mut close = false;
    let mut dev = "/dev/cdrom".to_string();
    for a in args {
        match a.as_str() {
            "-t" => close = true,
            s if !s.starts_with('-') => dev = s.to_string(),
            _ => {}
        }
    }

    let file = match std::fs::File::open(&dev) {
        Ok(f) => f,
        Err(e) => { eprintln!("eject: {dev}: {e}"); return 1; }
    };
    let req = if close { CDROMCLOSETRAY } else { CDROMEJECT };
    if unsafe { libc::ioctl(file.as_raw_fd(), req as _) } != 0 {
        eprintln!("eject: {dev}: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}
