/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! deluser — delete a user from the system

use std::fs;
use std::io::Write;

pub fn run(args: &[String]) -> i32 {
    let mut remove_home = false;
    let mut username = String::new();

    for arg in args {
        match arg.as_str() {
            "--remove-home" => remove_home = true,
            "-h" | "--help" => {
                eprintln!("Usage: deluser [--remove-home] USER");
                return 0;
            }
            s if !s.starts_with('-') => username = s.to_string(),
            other => {
                eprintln!("deluser: unknown option: {other}");
                return 1;
            }
        }
    }

    if username.is_empty() {
        eprintln!("Usage: deluser [--remove-home] USER");
        return 1;
    }

    // Read and filter /etc/passwd
    let passwd = match fs::read_to_string("/etc/passwd") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("deluser: cannot read /etc/passwd: {e}");
            return 1;
        }
    };

    let mut found = false;
    let mut home_dir = String::new();
    let mut new_passwd = String::new();
    for line in passwd.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if !parts.is_empty() && parts[0] == username {
            found = true;
            if parts.len() >= 6 {
                home_dir = parts[5].to_string();
            }
            continue;
        }
        new_passwd.push_str(line);
        new_passwd.push('\n');
    }

    if !found {
        eprintln!("deluser: user '{username}' does not exist");
        return 1;
    }

    // Write updated passwd
    match fs::File::create("/etc/passwd") {
        Ok(mut f) => {
            if let Err(e) = f.write_all(new_passwd.as_bytes()) {
                eprintln!("deluser: failed to write /etc/passwd: {e}");
                return 1;
            }
        }
        Err(e) => {
            eprintln!("deluser: cannot write /etc/passwd: {e}");
            return 1;
        }
    }

    // Remove from /etc/shadow
    if std::path::Path::new("/etc/shadow").exists() {
        if let Ok(shadow) = fs::read_to_string("/etc/shadow") {
            let new_shadow: String = shadow
                .lines()
                .filter(|l| l.split(':').next() != Some(username.as_str()))
                .map(|l| format!("{l}\n"))
                .collect();
            let _ = fs::write("/etc/shadow", new_shadow);
        }
    }

    // Remove from /etc/group membership
    if let Ok(groups) = fs::read_to_string("/etc/group") {
        let new_groups: String = groups
            .lines()
            .map(|line| {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let members: Vec<&str> = parts[3]
                        .split(',')
                        .filter(|m| !m.is_empty() && *m != username)
                        .collect();
                    format!("{}:{}:{}:{}", parts[0], parts[1], parts[2], members.join(","))
                } else {
                    line.to_string()
                }
            })
            .map(|l| format!("{l}\n"))
            .collect();
        let _ = fs::write("/etc/group", new_groups);
    }

    // Remove home directory if requested
    if remove_home && !home_dir.is_empty() {
        if let Err(e) = fs::remove_dir_all(&home_dir) {
            eprintln!("deluser: warning: cannot remove home '{home_dir}': {e}");
        }
    }

    println!("Removing user '{username}'");
    0
}
