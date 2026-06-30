/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! flash_lock / flash_unlock — lock or unlock MTD flash regions
use std::os::unix::io::AsRawFd;

// struct erase_info_user { __u32 start; __u32 length; }
#[repr(C)]
struct EraseInfo {
    start: u32,
    length: u32,
}

const MEMLOCK: libc::c_ulong = 0x40084d05;
const MEMUNLOCK: libc::c_ulong = 0x40084d06;

/// `lock` selects MEMLOCK vs MEMUNLOCK so the same code backs both applets.
pub fn run_with(args: &[String], lock: bool) -> i32 {
    let name = if lock { "flash_lock" } else { "flash_unlock" };
    let pos: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if pos.is_empty() {
        eprintln!("Usage: {name} MTD-DEVICE [OFFSET [BLOCK_COUNT]]");
        return 1;
    }
    let dev = pos[0].clone();
    let start: u32 = pos.get(1).and_then(|s| parse(s)).unwrap_or(0);
    // length of 0 / -1 means "to end of device".
    let length: u32 = pos.get(2).and_then(|s| parse(s)).unwrap_or(0);

    let file = match std::fs::OpenOptions::new().read(true).write(true).open(&dev) {
        Ok(f) => f,
        Err(e) => { eprintln!("{name}: {dev}: {e}"); return 1; }
    };

    let info = EraseInfo { start, length };
    let req = if lock { MEMLOCK } else { MEMUNLOCK };
    if unsafe { libc::ioctl(file.as_raw_fd(), req as _, &info) } != 0 {
        eprintln!("{name}: {dev}: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}

pub fn run(args: &[String]) -> i32 {
    run_with(args, true)
}

fn parse(s: &str) -> Option<u32> {
    if let Some(h) = s.strip_prefix("0x") {
        u32::from_str_radix(h, 16).ok()
    } else {
        s.parse().ok()
    }
}
