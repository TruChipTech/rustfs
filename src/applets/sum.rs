/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! sum — checksum and count the blocks in a file (BSD/SysV)
use std::fs::File;
use std::io::{self, Read};

pub fn run(args: &[String]) -> i32 {
    let mut sysv = false;
    let mut files = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-s" | "--sysv" => sysv = true,
            "-r" => {} // BSD is default
            _ => files.push(arg.clone()),
        }
    }
    if files.is_empty() {
        files.push("-".to_string());
    }

    let mut exit_code = 0;
    for file in &files {
        match read_all(file) {
            Ok(data) => {
                let (sum, blocks) = if sysv { sysv_sum(&data) } else { bsd_sum(&data) };
                if file == "-" {
                    println!("{sum} {blocks}");
                } else {
                    println!("{sum} {blocks} {file}");
                }
            }
            Err(e) => {
                eprintln!("sum: {file}: {e}");
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

fn bsd_sum(data: &[u8]) -> (u32, usize) {
    let mut checksum: u32 = 0;
    for &b in data {
        checksum = (checksum >> 1) | ((checksum & 1) << 15);
        checksum = (checksum + b as u32) & 0xffff;
    }
    (checksum, data.len().div_ceil(1024))
}

fn sysv_sum(data: &[u8]) -> (u32, usize) {
    let s: u32 = data.iter().map(|&b| b as u32).sum();
    let r = (s & 0xffff) + ((s >> 16) & 0xffff);
    let checksum = (r & 0xffff) + (r >> 16);
    (checksum, data.len().div_ceil(512))
}
