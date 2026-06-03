/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! hd — hexdump (canonical hex+ASCII display)

use std::fs;
use std::io::{self, Read};

pub fn run(args: &[String]) -> i32 {
    let mut files: Vec<String> = Vec::new();
    let mut length: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                i += 1;
                if i < args.len() { length = args[i].parse().ok(); }
            }
            "-h" | "--help" => {
                eprintln!("Usage: hd [-n LENGTH] [file...]");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    let data = if files.is_empty() {
        let mut buf = Vec::new();
        if let Some(len) = length {
            buf.resize(len, 0);
            let _ = io::stdin().read_exact(&mut buf);
        } else {
            let _ = io::stdin().read_to_end(&mut buf);
        }
        buf
    } else {
        let mut buf = Vec::new();
        for file in &files {
            match fs::read(file) {
                Ok(mut d) => buf.append(&mut d),
                Err(e) => {
                    eprintln!("hd: {file}: {e}");
                    return 1;
                }
            }
        }
        if let Some(len) = length {
            buf.truncate(len);
        }
        buf
    };

    hexdump_canonical(&data);
    0
}

fn hexdump_canonical(data: &[u8]) {
    for (offset, chunk) in data.chunks(16).enumerate() {
        print!("{:08x}  ", offset * 16);

        for (i, byte) in chunk.iter().enumerate() {
            print!("{byte:02x} ");
            if i == 7 { print!(" "); }
        }

        // Pad if less than 16 bytes
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
