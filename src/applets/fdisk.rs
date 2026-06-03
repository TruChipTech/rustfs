/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! fdisk — manipulate disk partition table

use std::fs;
use std::io::{self, Read, Write, BufRead};

pub fn run(args: &[String]) -> i32 {
    let mut list = false;
    let mut device = String::new();

    for arg in args {
        match arg.as_str() {
            "-l" | "--list" => list = true,
            "-h" | "--help" => {
                eprintln!("Usage: fdisk [-l] [device]");
                return 0;
            }
            s if !s.starts_with('-') => device = s.to_string(),
            _ => {}
        }
    }

    if list {
        if device.is_empty() {
            return list_all_disks();
        } else {
            return list_partitions(&device);
        }
    }

    if device.is_empty() {
        eprintln!("Usage: fdisk [-l] device");
        return 1;
    }

    interactive_mode(&device)
}

fn list_all_disks() -> i32 {
    let content = match fs::read_to_string("/proc/partitions") {
        Ok(c) => c,
        Err(e) => { eprintln!("fdisk: {e}"); return 1; }
    };

    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let name = parts[3];
            // Only show whole disks (no trailing digits typically, but heuristic)
            if !name.ends_with(|c: char| c.is_ascii_digit()) || name.starts_with("sd") || name.starts_with("vd") || name.starts_with("nvme") {
                let dev = format!("/dev/{name}");
                if std::path::Path::new(&dev).exists() {
                    let _ = list_partitions(&dev);
                    println!();
                }
            }
        }
    }
    0
}

fn list_partitions(device: &str) -> i32 {
    let mut file = match fs::File::open(device) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("fdisk: cannot open {device}: {e}");
            return 1;
        }
    };

    // Read MBR (first 512 bytes)
    let mut mbr = [0u8; 512];
    if file.read_exact(&mut mbr).is_err() {
        eprintln!("fdisk: cannot read {device}");
        return 1;
    }

    // Check MBR signature
    if mbr[510] != 0x55 || mbr[511] != 0xAA {
        eprintln!("fdisk: {device}: does not contain a valid partition table");
        return 1;
    }

    // Get disk size
    let disk_size = match fs::metadata(device) {
        Ok(m) => m.len(),
        Err(_) => 0,
    };

    let sectors = disk_size / 512;
    let size_gib = disk_size as f64 / (1024.0 * 1024.0 * 1024.0);
    println!("Disk {device}: {size_gib:.1} GiB, {disk_size} bytes, {sectors} sectors");
    println!("Sector size: 512 bytes");
    println!();
    println!("Device       Boot   Start       End   Sectors   Size  Id  Type");

    // Parse 4 primary partition entries (at offset 446, 16 bytes each)
    for i in 0..4 {
        let offset = 446 + i * 16;
        let entry = &mbr[offset..offset + 16];

        let status = entry[0];
        let ptype = entry[4];
        let lba_start = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]);
        let num_sectors = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]);

        if ptype == 0 || num_sectors == 0 { continue; }

        let boot = if status == 0x80 { "*" } else { " " };
        let lba_end = lba_start + num_sectors - 1;
        let size = (num_sectors as u64 * 512) as f64;
        let size_str = if size >= 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1}G", size / (1024.0 * 1024.0 * 1024.0))
        } else if size >= 1024.0 * 1024.0 {
            format!("{:.1}M", size / (1024.0 * 1024.0))
        } else {
            format!("{:.0}K", size / 1024.0)
        };

        let type_name = partition_type_name(ptype);
        println!("{device}{:<3}  {boot}  {lba_start:>8}  {lba_end:>8}  {num_sectors:>8}  {size_str:>5}  {ptype:02x}  {type_name}",
            i + 1);
    }

    0
}

fn interactive_mode(device: &str) -> i32 {
    println!("Welcome to fdisk ({device}).");
    println!("Changes will remain in memory only, until you decide to write them.");
    println!();
    println!("Command (m for help): ");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        match line.trim() {
            "m" => {
                println!("Command action");
                println!("   d   delete a partition");
                println!("   l   list known partition types");
                println!("   m   print this menu");
                println!("   n   add a new partition");
                println!("   p   print the partition table");
                println!("   q   quit without saving changes");
                println!("   t   change a partition's type");
                println!("   w   write table to disk and exit");
            }
            "p" => { let _ = list_partitions(device); }
            "q" => { return 0; }
            "w" => {
                println!("fdisk: write support not implemented for safety");
                return 0;
            }
            _ => println!("Unknown command: {}", line.trim()),
        }

        print!("Command (m for help): ");
        let _ = stdout.flush();
    }

    0
}

fn partition_type_name(ptype: u8) -> &'static str {
    match ptype {
        0x00 => "Empty",
        0x01 => "FAT12",
        0x04 => "FAT16 <32M",
        0x05 => "Extended",
        0x06 => "FAT16",
        0x07 => "HPFS/NTFS",
        0x0b => "W95 FAT32",
        0x0c => "W95 FAT32 (LBA)",
        0x0e => "W95 FAT16 (LBA)",
        0x0f => "W95 Ext'd (LBA)",
        0x11 => "Hidden FAT12",
        0x14 => "Hidden FAT16",
        0x16 => "Hidden FAT16",
        0x17 => "Hidden HPFS/NTFS",
        0x1b => "Hidden W95 FAT32",
        0x1c => "Hidden W95 FAT32",
        0x1e => "Hidden W95 FAT16",
        0x82 => "Linux swap",
        0x83 => "Linux",
        0x85 => "Linux extended",
        0x8e => "Linux LVM",
        0xee => "GPT",
        0xef => "EFI System",
        0xfd => "Linux raid",
        _ => "Unknown",
    }
}
