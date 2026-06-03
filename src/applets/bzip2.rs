/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! bzip2 — compress files using Burrows-Wheeler block sorting

use bzip2::write::BzEncoder;
use bzip2::Compression;
use std::fs;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut keep = false;
    let mut force = false;
    let mut stdout = false;
    let mut decompress = false;
    let mut level = 9u32;
    let mut files: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-k" | "--keep" => keep = true,
            "-f" | "--force" => force = true,
            "-c" | "--stdout" => stdout = true,
            "-d" | "--decompress" => decompress = true,
            "-1" | "--fast" => level = 1,
            "-9" | "--best" => level = 9,
            "-h" | "--help" => {
                eprintln!("Usage: bzip2 [-cdkf123456789] [file...]");
                return 0;
            }
            s if s.starts_with('-') && s.len() == 2 && s.as_bytes()[1].is_ascii_digit() => {
                level = (s.as_bytes()[1] - b'0') as u32;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
    }

    if decompress {
        #[cfg(applet_bunzip2)]
        return super::bunzip2::run(args);
        #[cfg(not(applet_bunzip2))]
        {
            eprintln!("bzip2: decompression not available");
            return 1;
        }
    }

    if files.is_empty() {
        // Compress stdin to stdout
        let mut buf = Vec::new();
        if io::stdin().read_to_end(&mut buf).is_err() {
            eprintln!("bzip2: error reading stdin");
            return 1;
        }
        return compress_data(&buf, &mut io::stdout(), level);
    }

    let mut exit_code = 0;
    for file in &files {
        if file.ends_with(".bz2") && !force {
            eprintln!("bzip2: {file} already has .bz2 suffix -- unchanged");
            continue;
        }

        let data = match fs::read(file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("bzip2: {file}: {e}");
                exit_code = 1;
                continue;
            }
        };

        let output_name = format!("{file}.bz2");

        if stdout {
            if compress_data(&data, &mut io::stdout(), level) != 0 {
                exit_code = 1;
            }
        } else {
            let out_file = match fs::File::create(&output_name) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("bzip2: {output_name}: {e}");
                    exit_code = 1;
                    continue;
                }
            };
            if compress_data(&data, &mut io::BufWriter::new(out_file), level) != 0 {
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

fn compress_data(data: &[u8], output: &mut dyn Write, level: u32) -> i32 {
    let mut encoder = BzEncoder::new(output, Compression::new(level));
    if let Err(e) = encoder.write_all(data) {
        eprintln!("bzip2: compression error: {e}");
        return 1;
    }
    if let Err(e) = encoder.finish() {
        eprintln!("bzip2: compression error: {e}");
        return 1;
    }
    0
}
