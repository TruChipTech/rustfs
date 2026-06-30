/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! chattr — change file attributes on a Linux ext2/3/4 file system
use crate::applets::lsattr::{get_flags, ATTR_ORDER};
use std::os::unix::io::AsRawFd;

const FS_IOC_SETFLAGS: libc::c_ulong = 0x40086602;

pub fn run(args: &[String]) -> i32 {
    let mut add: u32 = 0;
    let mut del: u32 = 0;
    let mut set: Option<u32> = None;
    let mut paths: Vec<String> = Vec::new();

    for a in args {
        if let Some(rest) = a.strip_prefix('+') {
            add |= letters_to_flags(rest);
        } else if let Some(rest) = a.strip_prefix('-') {
            // could be option like -R or attribute removal -i
            if rest.chars().all(|c| "RVf".contains(c)) && !rest.is_empty() {
                continue; // ignore -R/-V/-f
            }
            del |= letters_to_flags(rest);
        } else if let Some(rest) = a.strip_prefix('=') {
            set = Some(letters_to_flags(rest));
        } else {
            paths.push(a.clone());
        }
    }

    if paths.is_empty() {
        eprintln!("Usage: chattr [+-=][aAcdijsStT] FILE...");
        return 1;
    }

    let mut rc = 0;
    for p in &paths {
        let current = get_flags(p).unwrap_or(0);
        let new = match set {
            Some(s) => s,
            None => (current | add) & !del,
        };
        if set_flags(p, new).is_err() {
            eprintln!("chattr: {p}: {}", std::io::Error::last_os_error());
            rc = 1;
        }
    }
    rc
}

fn letters_to_flags(letters: &str) -> u32 {
    let mut f = 0;
    for c in letters.chars() {
        if let Some(&(bit, _)) = ATTR_ORDER.iter().find(|&&(_, ch)| ch == c) {
            f |= bit;
        }
    }
    f
}

fn set_flags(path: &str, flags: u32) -> std::io::Result<()> {
    let file = std::fs::File::open(path)?;
    let v = flags as libc::c_int;
    if unsafe { libc::ioctl(file.as_raw_fd(), FS_IOC_SETFLAGS as _, &v) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
