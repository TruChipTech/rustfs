/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! cksum — checksum and count the bytes in a file (POSIX CRC-32)
use std::fs::File;
use std::io::{self, Read};

pub fn run(args: &[String]) -> i32 {
    let table = build_table();
    let mut files: Vec<&String> = args.iter().filter(|a| !a.starts_with('-') || *a == "-").collect();
    let stdin_name = "-".to_string();
    if files.is_empty() {
        files.push(&stdin_name);
    }

    let mut exit_code = 0;
    for file in files {
        match read_all(file) {
            Ok(data) => {
                let crc = cksum(&data, &table);
                if file == "-" {
                    println!("{} {}", crc, data.len());
                } else {
                    println!("{} {} {}", crc, data.len(), file);
                }
            }
            Err(e) => {
                eprintln!("cksum: {file}: {e}");
                exit_code = 1;
            }
        }
    }
    exit_code
}

fn read_all(file: &str) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    if file == "-" {
        io::stdin().read_to_end(&mut data)?;
    } else {
        File::open(file)?.read_to_end(&mut data)?;
    }
    Ok(data)
}

fn build_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    for (i, slot) in t.iter_mut().enumerate() {
        let mut c = (i as u32) << 24;
        for _ in 0..8 {
            c = if c & 0x8000_0000 != 0 {
                (c << 1) ^ 0x04C1_1DB7
            } else {
                c << 1
            };
        }
        *slot = c;
    }
    t
}

fn cksum(data: &[u8], table: &[u32; 256]) -> u32 {
    let mut crc: u32 = 0;
    for &b in data {
        crc = (crc << 8) ^ table[(((crc >> 24) as u8) ^ b) as usize];
    }
    let mut len = data.len();
    while len > 0 {
        crc = (crc << 8) ^ table[(((crc >> 24) as u8) ^ (len as u8)) as usize];
        len >>= 8;
    }
    !crc
}
