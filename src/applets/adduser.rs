/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! adduser — add a user to the system

use std::fs;
use std::io::Write;
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut uid: Option<u32> = None;
    let mut gid: Option<u32> = None;
    let mut home: Option<String> = None;
    let mut shell = "/bin/sh".to_string();
    let mut gecos = String::new();
    let mut system_user = false;
    let mut no_create_home = false;
    let mut disabled_password = false;
    let mut username = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-u" | "--uid" => { i += 1; if i < args.len() { uid = args[i].parse().ok(); } }
            "-g" | "--gid" => { i += 1; if i < args.len() { gid = args[i].parse().ok(); } }
            "-h" | "--home" => { i += 1; if i < args.len() { home = Some(args[i].clone()); } }
            "-s" | "--shell" => { i += 1; if i < args.len() { shell = args[i].clone(); } }
            "-G" | "--gecos" => { i += 1; if i < args.len() { gecos = args[i].clone(); } }
            "-S" | "--system" => system_user = true,
            "-H" | "--no-create-home" => no_create_home = true,
            "-D" | "--disabled-password" => disabled_password = true,
            s if !s.starts_with('-') => username = s.to_string(),
            other => { eprintln!("adduser: unknown option: {other}"); return 1; }
        }
        i += 1;
    }

    if username.is_empty() {
        eprintln!("Usage: adduser [-u UID] [-g GID] [-h HOME] [-s SHELL] [-S] [-H] [-D] USER");
        return 1;
    }

    // Check if user exists
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            if let Some(name) = line.split(':').next() {
                if name == username {
                    eprintln!("adduser: user '{username}' already exists");
                    return 1;
                }
            }
        }
    }

    let assigned_uid = uid.unwrap_or_else(|| {
        let start = if system_user { 100 } else { 1000 };
        find_next_uid(start)
    });

    let assigned_gid = gid.unwrap_or(assigned_uid);
    let home_dir = home.unwrap_or_else(|| {
        if system_user { "/dev/null".to_string() } else { format!("/home/{username}") }
    });

    // Create group with same name if it doesn't exist
    if !group_exists(&username) {
        let group_entry = format!("{username}:x:{assigned_gid}:\n");
        if let Ok(mut f) = fs::OpenOptions::new().append(true).open("/etc/group") {
            let _ = f.write_all(group_entry.as_bytes());
        }
    }

    // Add to /etc/passwd
    let passwd_entry = format!("{username}:x:{assigned_uid}:{assigned_gid}:{gecos}:{home_dir}:{shell}\n");
    match fs::OpenOptions::new().append(true).open("/etc/passwd") {
        Ok(mut f) => {
            if let Err(e) = f.write_all(passwd_entry.as_bytes()) {
                eprintln!("adduser: failed to write /etc/passwd: {e}");
                return 1;
            }
        }
        Err(e) => {
            eprintln!("adduser: cannot open /etc/passwd: {e}");
            return 1;
        }
    }

    // Add to /etc/shadow
    let shadow_entry = if disabled_password {
        format!("{username}:!:0:0:99999:7:::\n")
    } else {
        format!("{username}:*:0:0:99999:7:::\n")
    };
    if let Ok(mut f) = fs::OpenOptions::new().append(true).open("/etc/shadow") {
        let _ = f.write_all(shadow_entry.as_bytes());
    }

    // Create home directory
    if !no_create_home && !system_user {
        let _ = fs::create_dir_all(&home_dir);
        // Copy skel if available
        if std::path::Path::new("/etc/skel").is_dir() {
            let _ = Command::new("/bin/cp")
                .args(["-a", "/etc/skel/.", &home_dir])
                .status();
        }
        // Set ownership
        let c_path = std::ffi::CString::new(home_dir.as_str()).ok();
        if let Some(p) = c_path {
            unsafe { libc::chown(p.as_ptr(), assigned_uid, assigned_gid) };
        }
    }

    println!("Adding user '{username}' (UID {assigned_uid})");
    0
}

fn find_next_uid(start: u32) -> u32 {
    let mut used = std::collections::HashSet::new();
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(u) = parts[2].parse::<u32>() {
                    used.insert(u);
                }
            }
        }
    }
    let mut id = start;
    while used.contains(&id) { id += 1; }
    id
}

fn group_exists(name: &str) -> bool {
    if let Ok(content) = fs::read_to_string("/etc/group") {
        content.lines().any(|l| l.split(':').next() == Some(name))
    } else {
        false
    }
}
