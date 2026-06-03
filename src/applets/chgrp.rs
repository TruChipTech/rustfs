/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! chgrp — change group ownership of files

use std::ffi::CString;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut recursive = false;
    let mut verbose = false;
    let mut group_name = String::new();
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-R" | "--recursive" => recursive = true,
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => {
                eprintln!("Usage: chgrp [-Rv] GROUP FILE...");
                return 0;
            }
            s if !s.starts_with('-') => {
                if group_name.is_empty() {
                    group_name = s.to_string();
                } else {
                    files.push(s.to_string());
                }
            }
            other => {
                eprintln!("chgrp: unknown option: {other}");
                return 1;
            }
        }
        i += 1;
    }

    if group_name.is_empty() || files.is_empty() {
        eprintln!("Usage: chgrp [-Rv] GROUP FILE...");
        return 1;
    }

    let gid = resolve_gid(&group_name);
    if gid.is_none() {
        eprintln!("chgrp: invalid group: '{group_name}'");
        return 1;
    }
    let gid = gid.unwrap();

    let mut exit_code = 0;
    for file in &files {
        if chgrp_file(file, gid, recursive, verbose) != 0 {
            exit_code = 1;
        }
    }
    exit_code
}

fn chgrp_file(path: &str, gid: u32, recursive: bool, verbose: bool) -> i32 {
    let c_path = match CString::new(path) {
        Ok(p) => p,
        Err(_) => {
            eprintln!("chgrp: invalid path: {path}");
            return 1;
        }
    };

    // Get current owner uid to preserve it
    let mut stat_buf: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::lstat(c_path.as_ptr(), &mut stat_buf) } != 0 {
        eprintln!("chgrp: cannot access '{path}': {}", std::io::Error::last_os_error());
        return 1;
    }

    let uid = stat_buf.st_uid;
    let result = unsafe { libc::chown(c_path.as_ptr(), uid, gid) };
    if result != 0 {
        eprintln!("chgrp: changing group of '{path}': {}", std::io::Error::last_os_error());
        return 1;
    }

    if verbose {
        println!("changed group of '{path}' to {gid}");
    }

    if recursive {
        if let Ok(md) = fs::symlink_metadata(path) {
            if md.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let child = entry.path().to_string_lossy().to_string();
                        chgrp_file(&child, gid, true, verbose);
                    }
                }
            }
        }
    }

    0
}

fn resolve_gid(group: &str) -> Option<u32> {
    // Try numeric GID first
    if let Ok(gid) = group.parse::<u32>() {
        return Some(gid);
    }

    // Look up in /etc/group
    if let Ok(content) = fs::read_to_string("/etc/group") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 && parts[0] == group {
                if let Ok(gid) = parts[2].parse::<u32>() {
                    return Some(gid);
                }
            }
        }
    }

    None
}
