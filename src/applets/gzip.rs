/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! gzip — compress files

use std::fs;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut keep = false;
    let mut force = false;
    let mut stdout = false;
    let mut decompress = false;
    let mut _level = 6;
    let mut files: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-k" | "--keep" => keep = true,
            "-f" | "--force" => force = true,
            "-c" | "--stdout" => stdout = true,
            "-d" | "--decompress" => decompress = true,
            "-1" | "--fast" => _level = 1,
            "-9" | "--best" => _level = 9,
            "-h" | "--help" => {
                eprintln!("Usage: gzip [-cdfk123456789] [file...]");
                return 0;
            }
            s if s.starts_with('-') && s.len() == 2 && s.as_bytes()[1].is_ascii_digit() => {
                _level = (s.as_bytes()[1] - b'0') as i32;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
    }

    if decompress {
        #[cfg(applet_gunzip)]
        return super::gunzip::run(args);
        #[cfg(not(applet_gunzip))]
        {
            eprintln!("gzip: decompression not available");
            return 1;
        }
    }

    if files.is_empty() {
        // Compress stdin to stdout using stored blocks
        let mut data = Vec::new();
        if io::stdin().read_to_end(&mut data).is_err() {
            eprintln!("gzip: error reading stdin");
            return 1;
        }
        let compressed = gzip_stored(&data);
        if let Err(e) = io::stdout().write_all(&compressed) {
            eprintln!("gzip: {e}");
            return 1;
        }
        return 0;
    }

    let mut exit_code = 0;
    for file in &files {
        if file.ends_with(".gz") && !force {
            eprintln!("gzip: {file} already has .gz suffix -- unchanged");
            continue;
        }

        let data = match fs::read(file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("gzip: {file}: {e}");
                exit_code = 1;
                continue;
            }
        };

        let compressed = gzip_stored(&data);

        if stdout {
            if let Err(e) = io::stdout().write_all(&compressed) {
                eprintln!("gzip: {e}");
                exit_code = 1;
            }
        } else {
            let output_name = format!("{file}.gz");
            if let Err(e) = fs::write(&output_name, &compressed) {
                eprintln!("gzip: {output_name}: {e}");
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

/// Create a gzip file using DEFLATE stored blocks (no compression, but valid gzip)
fn gzip_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();

    // Gzip header
    out.push(0x1f); // magic
    out.push(0x8b); // magic
    out.push(0x08); // method: deflate
    out.push(0x00); // flags
    out.extend_from_slice(&[0, 0, 0, 0]); // mtime
    out.push(0x00); // xfl
    out.push(0x03); // OS: Unix

    // DEFLATE stored blocks
    let mut offset = 0;
    while offset < data.len() {
        let remaining = data.len() - offset;
        let block_size = remaining.min(65535);
        let is_last = offset + block_size >= data.len();

        out.push(if is_last { 0x01 } else { 0x00 }); // BFINAL + BTYPE=00
        let len = block_size as u16;
        let nlen = !len;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&nlen.to_le_bytes());
        out.extend_from_slice(&data[offset..offset + block_size]);

        offset += block_size;
    }

    // Empty stored block if data is empty
    if data.is_empty() {
        out.push(0x01);
        out.extend_from_slice(&[0, 0, 0xff, 0xff]);
    }

    // CRC32 and size
    let crc = crc32(data);
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());

    out
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}
