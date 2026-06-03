/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! blkid — locate/print block device attributes

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut devices: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                eprintln!("Usage: blkid [device...]");
                return 0;
            }
            s => devices.push(s.to_string()),
        }
    }

    if devices.is_empty() {
        // Scan all block devices
        return scan_all_devices();
    }

    let mut exit_code = 0;
    for dev in &devices {
        if print_device_info(dev) != 0 {
            exit_code = 1;
        }
    }
    exit_code
}

fn scan_all_devices() -> i32 {
    // Read /proc/partitions to find block devices
    let content = match fs::read_to_string("/proc/partitions") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("blkid: cannot read /proc/partitions: {e}");
            return 1;
        }
    };

    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let name = parts[3];
            let dev_path = format!("/dev/{name}");
            if std::path::Path::new(&dev_path).exists() {
                let _ = print_device_info(&dev_path);
            }
        }
    }
    0
}

fn print_device_info(device: &str) -> i32 {
    // Try to read filesystem superblock to identify type and UUID
    let data = match fs::read(device) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("blkid: cannot open {device}: {e}");
            return 1;
        }
    };

    let mut fstype = String::new();
    let mut uuid = String::new();
    let mut label = String::new();

    // Check ext2/3/4 superblock at offset 1024
    if data.len() >= 1024 + 264 {
        let magic = u16::from_le_bytes([data[1024 + 56], data[1024 + 57]]);
        if magic == 0xEF53 {
            // ext2/3/4
            let compat = u32::from_le_bytes([data[1024 + 92], data[1024 + 93], data[1024 + 94], data[1024 + 95]]);
            let incompat = u32::from_le_bytes([data[1024 + 96], data[1024 + 97], data[1024 + 98], data[1024 + 99]]);

            if incompat & 0x0040 != 0 {
                fstype = "ext4".to_string();
            } else if compat & 0x0004 != 0 {
                fstype = "ext3".to_string();
            } else {
                fstype = "ext2".to_string();
            }

            // UUID at offset 1024+104, 16 bytes
            let u = &data[1024 + 104..1024 + 120];
            uuid = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7],
                u[8], u[9], u[10], u[11], u[12], u[13], u[14], u[15]
            );

            // Label at offset 1024+120, 16 bytes
            let lbl = &data[1024 + 120..1024 + 136];
            if let Ok(l) = std::str::from_utf8(lbl) {
                label = l.trim_end_matches('\0').to_string();
            }
        }
    }

    // Check for swap signature
    if fstype.is_empty() && data.len() >= 4096 + 10 {
        if &data[4086..4096] == b"SWAPSPACE2" || &data[4086..4096] == b"SWAP-SPACE" {
            fstype = "swap".to_string();
        }
    }

    if fstype.is_empty() {
        return 2; // Not identified
    }

    let mut output = format!("{device}:");
    if !uuid.is_empty() {
        output.push_str(&format!(" UUID=\"{uuid}\""));
    }
    if !label.is_empty() {
        output.push_str(&format!(" LABEL=\"{label}\""));
    }
    output.push_str(&format!(" TYPE=\"{fstype}\""));
    println!("{output}");

    0
}
