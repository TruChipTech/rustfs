/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! mountpoint — see if a directory is a mountpoint
use std::ffi::CString;

pub fn run(args: &[String]) -> i32 {
    let mut quiet = false;
    let mut show_dev = false;
    let mut path = None;

    for a in args {
        match a.as_str() {
            "-q" => quiet = true,
            "-d" => show_dev = true,
            "-x" => show_dev = true,
            s if !s.starts_with('-') => path = Some(s.to_string()),
            _ => {}
        }
    }

    let path = match path {
        Some(p) => p,
        None => { eprintln!("Usage: mountpoint [-q] [-d] DIR"); return 1; }
    };

    let st = match stat(&path) {
        Some(s) => s,
        None => {
            if !quiet { eprintln!("mountpoint: {path}: no such file or directory"); }
            return 1;
        }
    };

    if show_dev {
        println!("{}:{}", major(st.st_dev), minor(st.st_dev));
        return 0;
    }

    // A directory is a mountpoint if its device differs from its parent's,
    // or it is the root of its filesystem (dev differs but inode 2 / "..").
    let parent = format!("{path}/..");
    let pst = match stat(&parent) {
        Some(s) => s,
        None => return 1,
    };

    let is_mp = st.st_dev != pst.st_dev || st.st_ino == pst.st_ino;
    if is_mp {
        if !quiet { println!("{path} is a mountpoint"); }
        0
    } else {
        if !quiet { println!("{path} is not a mountpoint"); }
        1
    }
}

fn stat(path: &str) -> Option<libc::stat> {
    let c = CString::new(path).ok()?;
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::stat(c.as_ptr(), &mut st) } == 0 {
        Some(st)
    } else {
        None
    }
}

fn major(dev: libc::dev_t) -> u64 {
    ((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff)
}

fn minor(dev: libc::dev_t) -> u64 {
    (dev & 0xff) | ((dev >> 12) & !0xff)
}
