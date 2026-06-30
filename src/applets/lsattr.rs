/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! lsattr — list file attributes on a Linux ext2/3/4 file system
use std::os::unix::io::AsRawFd;

const FS_IOC_GETFLAGS: libc::c_ulong = 0x80086601;

// (flag bit, letter) in the canonical e2fsprogs display order.
pub const ATTR_ORDER: &[(u32, char)] = &[
    (0x00000004, 'S'), // sync updates
    (0x00000008, 's'), // secure deletion
    (0x00000010, 'i'), // immutable
    (0x00000020, 'a'), // append only
    (0x00000040, 'c'), // compress
    (0x00000200, 'd'), // no dump
    (0x00000400, 'A'), // no atime
    (0x00000800, 'j'), // data journalling
    (0x00001000, 't'), // no tail-merging
    (0x00020000, 'T'), // top of directory hierarchy
    (0x00080000, 'e'), // file uses extents
];

pub fn run(args: &[String]) -> i32 {
    let mut paths: Vec<String> = Vec::new();
    for a in args {
        if !a.starts_with('-') { paths.push(a.clone()); }
    }
    if paths.is_empty() { paths.push(".".to_string()); }

    let mut rc = 0;
    for p in &paths {
        let meta = match std::fs::metadata(p) {
            Ok(m) => m,
            Err(e) => { eprintln!("lsattr: {p}: {e}"); rc = 1; continue; }
        };
        if meta.is_dir() {
            if let Ok(rd) = std::fs::read_dir(p) {
                for e in rd.flatten() {
                    print_attrs(&e.path().to_string_lossy());
                }
            }
        } else {
            print_attrs(p);
        }
    }
    rc
}

fn print_attrs(path: &str) {
    match get_flags(path) {
        Some(flags) => println!("{} {}", format_flags(flags), path),
        None => eprintln!("lsattr: {path}: inappropriate ioctl for device"),
    }
}

pub fn get_flags(path: &str) -> Option<u32> {
    let file = std::fs::File::open(path).ok()?;
    let mut flags: libc::c_int = 0;
    if unsafe { libc::ioctl(file.as_raw_fd(), FS_IOC_GETFLAGS as _, &mut flags) } != 0 {
        return None;
    }
    Some(flags as u32)
}

fn format_flags(flags: u32) -> String {
    ATTR_ORDER.iter()
        .map(|&(bit, ch)| if flags & bit != 0 { ch } else { '-' })
        .collect()
}
