/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! hexdump — display file contents in hexadecimal

use std::fs;
use std::io::{self, Read};

pub fn run(args: &[String]) -> i32 {
    let mut canonical = false;
    let mut length: Option<usize> = None;
    let mut skip: usize = 0;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-C" => canonical = true,
            "-n" => {
                i += 1;
                if i < args.len() { length = args[i].parse().ok(); }
            }
            "-s" => {
                i += 1;
                if i < args.len() { skip = args[i].parse().unwrap_or(0); }
            }
            "-h" | "--help" => {
                eprintln!("Usage: hexdump [-C] [-n LENGTH] [-s SKIP] [file...]");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    let mut data = if files.is_empty() {
        let mut buf = Vec::new();
        let _ = io::stdin().read_to_end(&mut buf);
        buf
    } else {
        let mut buf = Vec::new();
        for file in &files {
            match fs::read(file) {
                Ok(mut d) => buf.append(&mut d),
                Err(e) => {
                    eprintln!("hexdump: {file}: {e}");
                    return 1;
                }
            }
        }
        buf
    };

    if skip > 0 && skip < data.len() {
        data = data[skip..].to_vec();
    }
    if let Some(len) = length {
        data.truncate(len);
    }

    if canonical {
        hexdump_canonical(&data);
    } else {
        hexdump_default(&data);
    }
    0
}

fn hexdump_canonical(data: &[u8]) {
    for (offset, chunk) in data.chunks(16).enumerate() {
        print!("{:08x}  ", offset * 16);
        for (i, byte) in chunk.iter().enumerate() {
            print!("{byte:02x} ");
            if i == 7 { print!(" "); }
        }
        if chunk.len() < 16 {
            for i in chunk.len()..16 {
                print!("   ");
                if i == 7 { print!(" "); }
            }
        }
        print!(" |");
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
    println!("{:08x}", data.len());
}

fn hexdump_default(data: &[u8]) {
    for (offset, chunk) in data.chunks(16).enumerate() {
        print!("{:07x}", offset * 16);
        for pair in chunk.chunks(2) {
            if pair.len() == 2 {
                print!(" {:02x}{:02x}", pair[1], pair[0]); // Little-endian word display
            } else {
                print!(" {:02x}  ", pair[0]);
            }
        }
        println!();
    }
    println!("{:07x}", data.len());
}
