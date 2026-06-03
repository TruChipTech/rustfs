/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! install — copy files and set attributes

use std::fs;
use std::os::unix::fs::PermissionsExt;

pub fn run(args: &[String]) -> i32 {
    let mut mode: u32 = 0o755;
    let mut owner: Option<String> = None;
    let mut group: Option<String> = None;
    let mut directory_mode = false;
    let mut strip = false;
    let mut sources: Vec<String> = Vec::new();
    let dest;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-m" | "--mode" => {
                i += 1;
                if i < args.len() { mode = parse_mode(&args[i]); }
            }
            "-o" | "--owner" => {
                i += 1;
                if i < args.len() { owner = Some(args[i].clone()); }
            }
            "-g" | "--group" => {
                i += 1;
                if i < args.len() { group = Some(args[i].clone()); }
            }
            "-d" | "--directory" => directory_mode = true,
            "-s" | "--strip" => strip = true,
            "-h" | "--help" => {
                eprintln!("Usage: install [-d] [-m MODE] [-o OWNER] [-g GROUP] [-s] SOURCE... DEST");
                return 0;
            }
            s if !s.starts_with('-') => sources.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    if directory_mode {
        // Create directories
        for dir in &sources {
            if let Err(e) = fs::create_dir_all(dir) {
                eprintln!("install: cannot create directory '{dir}': {e}");
                return 1;
            }
            let _ = fs::set_permissions(dir, fs::Permissions::from_mode(mode));
        }
        return 0;
    }

    if sources.len() < 2 {
        eprintln!("Usage: install [-d] [-m MODE] [-o OWNER] [-g GROUP] SOURCE... DEST");
        return 1;
    }

    dest = sources.pop().unwrap();
    let dest_is_dir = std::path::Path::new(&dest).is_dir();

    if sources.len() > 1 && !dest_is_dir {
        eprintln!("install: target '{dest}' is not a directory");
        return 1;
    }

    let mut exit_code = 0;
    for source in &sources {
        let target = if dest_is_dir {
            let filename = std::path::Path::new(source)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(source);
            format!("{dest}/{filename}")
        } else {
            dest.clone()
        };

        // Copy the file
        if let Err(e) = fs::copy(source, &target) {
            eprintln!("install: cannot copy '{source}' to '{target}': {e}");
            exit_code = 1;
            continue;
        }

        // Set mode
        if let Err(e) = fs::set_permissions(&target, fs::Permissions::from_mode(mode)) {
            eprintln!("install: cannot set mode on '{target}': {e}");
        }

        // Set ownership
        if owner.is_some() || group.is_some() {
            let uid = owner.as_ref().and_then(|o| resolve_uid(o));
            let gid = group.as_ref().and_then(|g| resolve_gid(g));

            let c_path = std::ffi::CString::new(target.as_str()).unwrap();
            unsafe {
                libc::chown(
                    c_path.as_ptr(),
                    uid.unwrap_or(u32::MAX),
                    gid.unwrap_or(u32::MAX),
                );
            }
        }

        // Strip if requested
        if strip {
            let _ = std::process::Command::new("strip").arg(&target).status();
        }
    }

    exit_code
}

fn parse_mode(s: &str) -> u32 {
    u32::from_str_radix(s, 8).unwrap_or(0o755)
}

fn resolve_uid(user: &str) -> Option<u32> {
    if let Ok(uid) = user.parse::<u32>() { return Some(uid); }
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 && parts[0] == user {
                return parts[2].parse().ok();
            }
        }
    }
    None
}

fn resolve_gid(group: &str) -> Option<u32> {
    if let Ok(gid) = group.parse::<u32>() { return Some(gid); }
    if let Ok(content) = fs::read_to_string("/etc/group") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 && parts[0] == group {
                return parts[2].parse().ok();
            }
        }
    }
    None
}
