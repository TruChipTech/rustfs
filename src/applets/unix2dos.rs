/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! unix2dos — convert Unix line endings to DOS

use std::fs;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut files: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                eprintln!("Usage: unix2dos [file...]");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
    }

    if files.is_empty() {
        let mut input = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut input) {
            eprintln!("unix2dos: {e}");
            return 1;
        }
        let output = convert(&input);
        if let Err(e) = io::stdout().write_all(&output) {
            eprintln!("unix2dos: {e}");
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
                        eprintln!("unix2dos: {file}: {e}");
                        exit_code = 1;
                    } else {
                        eprintln!("unix2dos: converting file {file} to DOS format...");
                    }
                }
            }
            Err(e) => {
                eprintln!("unix2dos: {file}: {e}");
                exit_code = 1;
            }
        }
    }
    exit_code
}

fn convert(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() + data.len() / 40);
    let mut prev = 0u8;
    for &byte in data {
        if byte == b'\n' && prev != b'\r' {
            result.push(b'\r');
        }
        result.push(byte);
        prev = byte;
    }
    result
}
