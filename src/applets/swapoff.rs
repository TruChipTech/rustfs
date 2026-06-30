/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! swapoff — disable devices and files for paging and swapping
use std::ffi::CString;

pub fn run(args: &[String]) -> i32 {
    let mut all = false;
    let mut devices: Vec<String> = Vec::new();
    for a in args {
        match a.as_str() {
            "-a" | "--all" => all = true,
            s if !s.starts_with('-') => devices.push(s.to_string()),
            _ => {}
        }
    }

    if all {
        if let Ok(content) = std::fs::read_to_string("/proc/swaps") {
            for line in content.lines().skip(1) {
                if let Some(dev) = line.split_whitespace().next() {
                    devices.push(dev.to_string());
                }
            }
        }
    }
    if devices.is_empty() {
        eprintln!("Usage: swapoff [-a] [DEVICE...]");
        return 1;
    }

    let mut rc = 0;
    for dev in &devices {
        let c = match CString::new(dev.as_str()) { Ok(c) => c, Err(_) => { rc = 1; continue; } };
        if unsafe { libc::swapoff(c.as_ptr()) } != 0 {
            eprintln!("swapoff: {dev}: {}", std::io::Error::last_os_error());
            rc = 1;
        }
    }
    rc
}
