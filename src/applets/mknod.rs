/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! mknod — make block or character special files
use std::ffi::CString;

pub fn run(args: &[String]) -> i32 {
    let mut mode: u32 = 0o666;
    let mut pos: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "-m" || a == "--mode" {
            i += 1;
            if let Some(m) = args.get(i) {
                mode = u32::from_str_radix(m.trim_start_matches("0o"), 8).unwrap_or(0o666);
            }
        } else if let Some(m) = a.strip_prefix("-m") {
            mode = u32::from_str_radix(m, 8).unwrap_or(0o666);
        } else {
            pos.push(a.clone());
        }
        i += 1;
    }

    if pos.len() < 2 {
        eprintln!("Usage: mknod [-m MODE] NAME TYPE [MAJOR MINOR]");
        return 1;
    }
    let name = &pos[0];
    let typ = &pos[1];

    let (fmt, need_dev) = match typ.as_str() {
        "b" => (libc::S_IFBLK, true),
        "c" | "u" => (libc::S_IFCHR, true),
        "p" => (libc::S_IFIFO, false),
        _ => { eprintln!("mknod: invalid type '{typ}'"); return 1; }
    };

    let dev = if need_dev {
        if pos.len() < 4 {
            eprintln!("mknod: missing major/minor for type '{typ}'");
            return 1;
        }
        let major: u64 = pos[2].parse().unwrap_or(0);
        let minor: u64 = pos[3].parse().unwrap_or(0);
        libc::makedev(major as libc::c_uint, minor as libc::c_uint)
    } else {
        0
    };

    let c = match CString::new(name.as_str()) { Ok(c) => c, Err(_) => return 1 };
    if unsafe { libc::mknod(c.as_ptr(), fmt | mode, dev) } != 0 {
        eprintln!("mknod: {name}: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}
