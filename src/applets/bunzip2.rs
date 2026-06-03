/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! bunzip2 — decompress bzip2 files

use bzip2::read::BzDecoder;
use std::fs;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut keep = false;
    let mut force = false;
    let mut stdout = false;
    let mut files: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-k" | "--keep" => keep = true,
            "-f" | "--force" => force = true,
            "-c" | "--stdout" => stdout = true,
            "-d" | "--decompress" => {} // already decompressing
            "-h" | "--help" => {
                eprintln!("Usage: bunzip2 [-cfk] [file...]");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
    }

    if files.is_empty() {
        return decompress_stream(&mut io::stdin(), &mut io::stdout());
    }

    let mut exit_code = 0;
    for file in &files {
        if !file.ends_with(".bz2") && !force {
            eprintln!("bunzip2: {file}: unknown suffix -- ignored");
            exit_code = 1;
            continue;
        }

        let output_name = file.trim_end_matches(".bz2").to_string();

        let data = match fs::read(file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("bunzip2: {file}: {e}");
                exit_code = 1;
                continue;
            }
        };

        if stdout {
            if decompress_stream(&mut data.as_slice(), &mut io::stdout()) != 0 {
                eprintln!("bunzip2: {file}: decompression failed");
                exit_code = 1;
            }
        } else {
            let out_file = match fs::File::create(&output_name) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("bunzip2: {output_name}: {e}");
                    exit_code = 1;
                    continue;
                }
            };
            let mut buf_out = io::BufWriter::new(out_file);
            if decompress_stream(&mut data.as_slice(), &mut buf_out) != 0 {
                eprintln!("bunzip2: {file}: decompression failed");
                let _ = fs::remove_file(&output_name);
                exit_code = 1;
                continue;
            }
            if !keep {
                let _ = fs::remove_file(file);
            }
        }
    }
    exit_code
}

fn decompress_stream(input: &mut dyn Read, output: &mut dyn Write) -> i32 {
    let mut decoder = BzDecoder::new(input);
    let mut buf = [0u8; 8192];
    loop {
        match decoder.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Err(e) = output.write_all(&buf[..n]) {
                    eprintln!("bunzip2: write error: {e}");
                    return 1;
                }
            }
            Err(e) => {
                eprintln!("bunzip2: decompression error: {e}");
                return 1;
            }
        }
    }
    0
}
