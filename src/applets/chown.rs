/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! chown — change file owner and group

use std::ffi::CString;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut recursive = false;
    let mut verbose = false;
    let mut owner_spec = String::new();
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-R" | "--recursive" => recursive = true,
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => {
                eprintln!("Usage: chown [-Rv] OWNER[:GROUP] FILE...");
                return 0;
            }
            s if !s.starts_with('-') => {
                if owner_spec.is_empty() {
                    owner_spec = s.to_string();
                } else {
                    files.push(s.to_string());
                }
            }
            other => {
                eprintln!("chown: unknown option: {other}");
                return 1;
            }
        }
        i += 1;
    }

    if owner_spec.is_empty() || files.is_empty() {
        eprintln!("Usage: chown [-Rv] OWNER[:GROUP] FILE...");
        return 1;
    }

    // Parse owner:group
    let (uid, gid) = parse_owner_group(&owner_spec);
    if uid.is_none() && gid.is_none() {
        eprintln!("chown: invalid user: '{owner_spec}'");
        return 1;
    }

    let mut exit_code = 0;
    for file in &files {
        if chown_file(file, uid, gid, recursive, verbose) != 0 {
            exit_code = 1;
        }
    }
    exit_code
}

fn chown_file(path: &str, uid: Option<u32>, gid: Option<u32>, recursive: bool, verbose: bool) -> i32 {
    let c_path = match CString::new(path) {
        Ok(p) => p,
        Err(_) => { eprintln!("chown: invalid path: {path}"); return 1; }
    };

    // Get current stat to preserve uid/gid if not changing
    let mut stat_buf: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::lstat(c_path.as_ptr(), &mut stat_buf) } != 0 {
        eprintln!("chown: cannot access '{path}': {}", std::io::Error::last_os_error());
        return 1;
    }

    let new_uid = uid.unwrap_or(stat_buf.st_uid);
    let new_gid = gid.unwrap_or(stat_buf.st_gid);

    let result = unsafe { libc::chown(c_path.as_ptr(), new_uid, new_gid) };
    if result != 0 {
        eprintln!("chown: changing ownership of '{path}': {}", std::io::Error::last_os_error());
        return 1;
    }

    if verbose {
        println!("changed ownership of '{path}' to {new_uid}:{new_gid}");
    }

    if recursive {
        if let Ok(md) = fs::symlink_metadata(path) {
            if md.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let child = entry.path().to_string_lossy().to_string();
                        chown_file(&child, uid, gid, true, verbose);
                    }
                }
            }
        }
    }

    0
}

fn parse_owner_group(spec: &str) -> (Option<u32>, Option<u32>) {
    let (user_part, group_part) = if let Some(pos) = spec.find(':') {
        let (u, g) = spec.split_at(pos);
        (if u.is_empty() { None } else { Some(u) }, if g.len() > 1 { Some(&g[1..]) } else { None })
    } else if let Some(pos) = spec.find('.') {
        let (u, g) = spec.split_at(pos);
        (if u.is_empty() { None } else { Some(u) }, if g.len() > 1 { Some(&g[1..]) } else { None })
    } else {
        (Some(spec), None)
    };

    let uid = user_part.and_then(resolve_uid);
    let gid = group_part.and_then(resolve_gid);

    (uid, gid)
}

fn resolve_uid(user: &str) -> Option<u32> {
    if let Ok(uid) = user.parse::<u32>() {
        return Some(uid);
    }
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 && parts[0] == user {
                if let Ok(uid) = parts[2].parse::<u32>() {
                    return Some(uid);
                }
            }
        }
    }
    None
}

fn resolve_gid(group: &str) -> Option<u32> {
    if let Ok(gid) = group.parse::<u32>() {
        return Some(gid);
    }
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
