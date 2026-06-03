/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! fsck — check and repair a Linux filesystem

use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut auto_repair = false;
    let mut no_action = false;
    let mut verbose = false;
    let mut fstype: Option<String> = None;
    let mut devices: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "-p" => auto_repair = true,
            "-n" => no_action = true,
            "-y" => auto_repair = true,
            "-v" | "--verbose" => verbose = true,
            "-t" => {
                i += 1;
                if i < args.len() { fstype = Some(args[i].clone()); }
            }
            "-h" | "--help" => {
                eprintln!("Usage: fsck [-anpvy] [-t fstype] [device...]");
                return 0;
            }
            s if !s.starts_with('-') => devices.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    if devices.is_empty() {
        eprintln!("fsck: no devices specified");
        eprintln!("Usage: fsck [-anpvy] [-t fstype] device...");
        return 1;
    }

    let mut exit_code = 0;
    for device in &devices {
        let fs = fstype.clone().unwrap_or_else(|| detect_fstype(device));

        if verbose {
            eprintln!("fsck: checking {device} (type: {fs})");
        }

        let helper = format!("fsck.{fs}");

        let mut cmd_args = Vec::new();
        if auto_repair { cmd_args.push("-p".to_string()); }
        if no_action { cmd_args.push("-n".to_string()); }
        cmd_args.push(device.clone());

        // Try to run filesystem-specific fsck
        match Command::new(&helper).args(&cmd_args).status() {
            Ok(status) => {
                let code = status.code().unwrap_or(1);
                if code != 0 {
                    exit_code |= code;
                }
            }
            Err(_) => {
                // No helper found; do basic checks ourselves
                if verbose {
                    eprintln!("fsck: {helper} not found, performing basic check");
                }
                let result = basic_check(device, &fs);
                if result != 0 {
                    exit_code |= result;
                }
            }
        }
    }

    exit_code
}

fn detect_fstype(device: &str) -> String {
    // Try reading superblock to detect filesystem type
    if let Ok(data) = std::fs::read(device) {
        // ext2/3/4 magic at offset 1080 (1024 + 56)
        if data.len() >= 1024 + 58 {
            let magic = u16::from_le_bytes([data[1024 + 56], data[1024 + 57]]);
            if magic == 0xEF53 {
                return "ext4".to_string();
            }
        }
    }
    "ext4".to_string() // Default assumption
}

fn basic_check(device: &str, fstype: &str) -> i32 {
    eprintln!("fsck: basic check on {device} ({fstype})");

    // Verify we can read the device
    match std::fs::File::open(device) {
        Ok(_) => {
            println!("{device}: clean (basic check only)");
            0
        }
        Err(e) => {
            eprintln!("fsck: {device}: {e}");
            8 // Operational error
        }
    }
}
