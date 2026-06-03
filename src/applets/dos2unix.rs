/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! dos2unix — convert DOS line endings to Unix

use std::fs;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut files: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                eprintln!("Usage: dos2unix [file...]");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
    }

    if files.is_empty() {
        // Process stdin to stdout
        let mut input = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut input) {
            eprintln!("dos2unix: {e}");
            return 1;
        }
        let output = convert(&input);
        if let Err(e) = io::stdout().write_all(&output) {
            eprintln!("dos2unix: {e}");
            return 1;
        }
        return 0;
    }

    let mut exit_code = 0;
    for file in &files {
        match fs::read(file) {
            Ok(data) => {
                let converted = convert(&data);
                if converted != data {
                    if let Err(e) = fs::write(file, &converted) {
                        eprintln!("dos2unix: {file}: {e}");
                        exit_code = 1;
                    } else {
                        eprintln!("dos2unix: converting file {file} to Unix format...");
                    }
                }
            }
            Err(e) => {
                eprintln!("dos2unix: {file}: {e}");
                exit_code = 1;
            }
        }
    }
    exit_code
}

fn convert(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    for &byte in data {
        if byte != b'\r' {
            result.push(byte);
        }
        // Skip \r — this handles both \r\n -> \n and lone \r removal
    }
    result
}
