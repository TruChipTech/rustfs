/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! delgroup — delete a group from the system

use std::fs;
use std::io::Write;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: delgroup GROUP");
        return 1;
    }

    let group_name = &args[0];

    // Read /etc/group, filter out the target
    let content = match fs::read_to_string("/etc/group") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("delgroup: cannot read /etc/group: {e}");
            return 1;
        }
    };

    let mut found = false;
    let mut new_content = String::new();
    for line in content.lines() {
        if let Some(name) = line.split(':').next() {
            if name == group_name {
                found = true;
                continue;
            }
        }
        new_content.push_str(line);
        new_content.push('\n');
    }

    if !found {
        eprintln!("delgroup: group '{group_name}' does not exist");
        return 1;
    }

    match fs::File::create("/etc/group") {
        Ok(mut f) => {
            if let Err(e) = f.write_all(new_content.as_bytes()) {
                eprintln!("delgroup: failed to write /etc/group: {e}");
                return 1;
            }
        }
        Err(e) => {
            eprintln!("delgroup: cannot write /etc/group: {e}");
            return 1;
        }
    }

    // Also remove from /etc/gshadow if it exists
    if std::path::Path::new("/etc/gshadow").exists() {
        if let Ok(shadow) = fs::read_to_string("/etc/gshadow") {
            let new_shadow: String = shadow
                .lines()
                .filter(|l| l.split(':').next() != Some(group_name.as_str()))
                .map(|l| format!("{l}\n"))
                .collect();
            let _ = fs::write("/etc/gshadow", new_shadow);
        }
    }

    println!("Removing group '{group_name}'");
    0
}
