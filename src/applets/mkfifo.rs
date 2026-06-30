/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! mkfifo — make FIFOs (named pipes)
use std::ffi::CString;

pub fn run(args: &[String]) -> i32 {
    let mut mode: libc::mode_t = 0o666;
    let mut names: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "-m" || a == "--mode" {
            i += 1;
            if let Some(m) = args.get(i) {
                mode = u32::from_str_radix(m, 8).unwrap_or(0o666) as libc::mode_t;
            }
        } else if let Some(m) = a.strip_prefix("-m") {
            mode = u32::from_str_radix(m, 8).unwrap_or(0o666) as libc::mode_t;
        } else if !a.starts_with('-') {
            names.push(a.clone());
        }
        i += 1;
    }

    if names.is_empty() {
        eprintln!("Usage: mkfifo [-m MODE] NAME...");
        return 1;
    }

    let mut rc = 0;
    for name in &names {
        let c = match CString::new(name.as_str()) { Ok(c) => c, Err(_) => { rc = 1; continue; } };
        if unsafe { libc::mkfifo(c.as_ptr(), mode) } != 0 {
            eprintln!("mkfifo: {name}: {}", std::io::Error::last_os_error());
            rc = 1;
        }
    }
    rc
}
