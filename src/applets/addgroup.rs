// Author: Anand | Company: Truchip | Date: 2026-06-03
//! addgroup — add a group to the system

use std::fs;
use std::io::Write;

pub fn run(args: &[String]) -> i32 {
    let mut gid: Option<u32> = None;
    let mut system_group = false;
    let mut group_name = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-g" | "--gid" => {
                i += 1;
                if i < args.len() {
                    gid = args[i].parse().ok();
                }
            }
            "-S" | "--system" => system_group = true,
            s if !s.starts_with('-') => group_name = s.to_string(),
            other => {
                eprintln!("addgroup: unknown option: {other}");
                return 1;
            }
        }
        i += 1;
    }

    if group_name.is_empty() {
        eprintln!("Usage: addgroup [-g GID] [-S] GROUP");
        return 1;
    }

    // Check if group already exists
    if let Ok(content) = fs::read_to_string("/etc/group") {
        for line in content.lines() {
            if let Some(name) = line.split(':').next() {
                if name == group_name {
                    eprintln!("addgroup: group '{group_name}' already exists");
                    return 1;
                }
            }
        }
    }

    // Find next available GID
    let assigned_gid = if let Some(g) = gid {
        g
    } else {
        let start = if system_group { 100 } else { 1000 };
        find_next_gid(start)
    };

    // Append to /etc/group
    let entry = format!("{group_name}:x:{assigned_gid}:\n");
    match fs::OpenOptions::new().append(true).open("/etc/group") {
        Ok(mut f) => {
            if let Err(e) = f.write_all(entry.as_bytes()) {
                eprintln!("addgroup: failed to write /etc/group: {e}");
                return 1;
            }
        }
        Err(e) => {
            eprintln!("addgroup: cannot open /etc/group: {e}");
            return 1;
        }
    }

    // Also add to /etc/gshadow if it exists
    if std::path::Path::new("/etc/gshadow").exists() {
        let shadow_entry = format!("{group_name}:!::\n");
        if let Ok(mut f) = fs::OpenOptions::new().append(true).open("/etc/gshadow") {
            let _ = f.write_all(shadow_entry.as_bytes());
        }
    }

    println!("Adding group '{group_name}' (GID {assigned_gid})");
    0
}

fn find_next_gid(start: u32) -> u32 {
    let mut used_gids = std::collections::HashSet::new();
    if let Ok(content) = fs::read_to_string("/etc/group") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(g) = parts[2].parse::<u32>() {
                    used_gids.insert(g);
                }
            }
        }
    }
    let mut gid = start;
    while used_gids.contains(&gid) {
        gid += 1;
    }
    gid
}
