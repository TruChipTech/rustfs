/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut decode = false;
    let mut wrap: usize = 76;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--decode" => decode = true,
            "-w" | "--wrap" => {
                i += 1;
                if i < args.len() {
                    wrap = args[i].parse().unwrap_or(76);
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let mut input = Vec::new();
    if files.is_empty() || (files.len() == 1 && files[0] == "-") {
        if let Err(e) = io::stdin().read_to_end(&mut input) {
            eprintln!("base64: read error: {e}");
            return 1;
        }
    } else {
        for f in &files {
            match std::fs::read(f) {
                Ok(data) => input.extend(data),
                Err(e) => {
                    eprintln!("base64: {f}: {e}");
                    return 1;
                }
            }
        }
    }

    if decode {
        // Remove whitespace before decoding
        let cleaned: String = String::from_utf8_lossy(&input)
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        match STANDARD.decode(&cleaned) {
            Ok(decoded) => {
                let _ = io::stdout().write_all(&decoded);
            }
            Err(e) => {
                eprintln!("base64: invalid input: {e}");
                return 1;
            }
        }
    } else {
        let encoded = STANDARD.encode(&input);
        if wrap == 0 {
            println!("{encoded}");
        } else {
            for chunk in encoded.as_bytes().chunks(wrap) {
                println!("{}", String::from_utf8_lossy(chunk));
            }
        }
    }

    0
}
