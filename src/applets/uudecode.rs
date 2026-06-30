/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! uudecode — decode a file created by uuencode
use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut out_override: Option<String> = None;
    let mut input = "-".to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => { i += 1; out_override = args.get(i).cloned(); }
            s if !s.starts_with('-') || s == "-" => input = s.to_string(),
            _ => {}
        }
        i += 1;
    }

    let reader: Box<dyn BufRead> = if input == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        match File::open(&input) {
            Ok(f) => Box::new(BufReader::new(f)),
            Err(e) => { eprintln!("uudecode: {input}: {e}"); return 1; }
        }
    };

    let mut lines = reader.lines().map_while(Result::ok);
    let header = loop {
        match lines.next() {
            Some(l) if l.starts_with("begin-base64 ") || l.starts_with("begin ") => break l,
            Some(_) => continue,
            None => { eprintln!("uudecode: no 'begin' line found"); return 1; }
        }
    };
    let base64_mode = header.starts_with("begin-base64");
    let name = out_override.unwrap_or_else(|| {
        header.split_whitespace().nth(2).unwrap_or("/dev/stdout").to_string()
    });

    let mut out: Box<dyn Write> = if name == "-" || name == "/dev/stdout" {
        Box::new(io::stdout())
    } else {
        match File::create(&name) {
            Ok(f) => Box::new(f),
            Err(e) => { eprintln!("uudecode: {name}: {e}"); return 1; }
        }
    };

    if base64_mode {
        let mut enc = String::new();
        for line in lines {
            if line.starts_with("====") { break; }
            enc.push_str(line.trim());
        }
        match STANDARD.decode(enc.as_bytes()) {
            Ok(d) => { let _ = out.write_all(&d); }
            Err(e) => { eprintln!("uudecode: bad base64: {e}"); return 1; }
        }
    } else {
        for line in lines {
            if line == "end" || line.is_empty() { break; }
            let bytes = line.as_bytes();
            let count = (bytes[0].wrapping_sub(b' ')) & 0x3f;
            if count == 0 { continue; }
            let mut decoded = Vec::new();
            for chunk in bytes[1..].chunks(4) {
                let mut vals = [0u8; 4];
                for (j, &c) in chunk.iter().enumerate() {
                    vals[j] = c.wrapping_sub(b' ') & 0x3f;
                }
                decoded.push((vals[0] << 2) | (vals[1] >> 4));
                decoded.push((vals[1] << 4) | (vals[2] >> 2));
                decoded.push((vals[2] << 6) | vals[3]);
            }
            decoded.truncate(count as usize);
            let _ = out.write_all(&decoded);
        }
    }
    0
}
