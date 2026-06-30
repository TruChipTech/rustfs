/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! swapon — enable devices and files for paging and swapping
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
        devices.extend(fstab_swaps());
    }
    if devices.is_empty() {
        // Print active swaps.
        if let Ok(s) = std::fs::read_to_string("/proc/swaps") {
            print!("{s}");
        }
        return 0;
    }

    let mut rc = 0;
    for dev in &devices {
        let c = match CString::new(dev.as_str()) { Ok(c) => c, Err(_) => { rc = 1; continue; } };
        if unsafe { libc::swapon(c.as_ptr(), 0) } != 0 {
            eprintln!("swapon: {dev}: {}", std::io::Error::last_os_error());
            rc = 1;
        }
    }
    rc
}

fn fstab_swaps() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(content) = std::fs::read_to_string("/etc/fstab") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.len() >= 3 && f[2] == "swap" {
                out.push(f[0].to_string());
            }
        }
    }
    out
}
