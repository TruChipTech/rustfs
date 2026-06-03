/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! gunzip — decompress gzip files

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
            "-h" | "--help" => {
                eprintln!("Usage: gunzip [-cfk] [file...]");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
    }

    if files.is_empty() {
        return decompress_stdin();
    }

    let mut exit_code = 0;
    for file in &files {
        if !file.ends_with(".gz") && !force {
            eprintln!("gunzip: {file}: unknown suffix -- ignored");
            exit_code = 1;
            continue;
        }

        let data = match fs::read(file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("gunzip: {file}: {e}");
                exit_code = 1;
                continue;
            }
        };

        // Verify gzip magic: 0x1f 0x8b
        if data.len() < 10 || data[0] != 0x1f || data[1] != 0x8b {
            eprintln!("gunzip: {file}: not in gzip format");
            exit_code = 1;
            continue;
        }

        let decompressed = match inflate_gzip(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("gunzip: {file}: {e}");
                exit_code = 1;
                continue;
            }
        };

        if stdout {
            if let Err(e) = io::stdout().write_all(&decompressed) {
                eprintln!("gunzip: write error: {e}");
                exit_code = 1;
            }
        } else {
            let output_name = file.trim_end_matches(".gz");
            if let Err(e) = fs::write(output_name, &decompressed) {
                eprintln!("gunzip: {output_name}: {e}");
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

fn decompress_stdin() -> i32 {
    let mut data = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut data) {
        eprintln!("gunzip: {e}");
        return 1;
    }
    if data.len() < 10 || data[0] != 0x1f || data[1] != 0x8b {
        eprintln!("gunzip: stdin: not in gzip format");
        return 1;
    }
    match inflate_gzip(&data) {
        Ok(d) => {
            if let Err(e) = io::stdout().write_all(&d) {
                eprintln!("gunzip: {e}");
                return 1;
            }
            0
        }
        Err(e) => {
            eprintln!("gunzip: {e}");
            1
        }
    }
}

/// Minimal DEFLATE decompressor for stored (uncompressed) blocks.
/// Full deflate decoding is complex; this handles common cases.
fn inflate_gzip(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 18 {
        return Err("truncated gzip".to_string());
    }

    let method = data[2];
    if method != 8 {
        return Err(format!("unsupported compression method: {method}"));
    }

    let flags = data[3];
    let mut offset = 10;

    // Skip extra field
    if flags & 0x04 != 0 {
        if offset + 2 > data.len() { return Err("truncated".to_string()); }
        let xlen = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2 + xlen;
    }
    // Skip original filename
    if flags & 0x08 != 0 {
        while offset < data.len() && data[offset] != 0 { offset += 1; }
        offset += 1;
    }
    // Skip comment
    if flags & 0x10 != 0 {
        while offset < data.len() && data[offset] != 0 { offset += 1; }
        offset += 1;
    }
    // Skip header CRC
    if flags & 0x02 != 0 {
        offset += 2;
    }

    if offset >= data.len() {
        return Err("truncated gzip data".to_string());
    }

    // The compressed data is DEFLATE format from offset to data.len()-8
    // For a minimal implementation, we use raw inflate
    let compressed = &data[offset..data.len().saturating_sub(8)];
    inflate_raw(compressed)
}

fn inflate_raw(data: &[u8]) -> Result<Vec<u8>, String> {
    // Minimal inflate: handle stored blocks only
    let mut output = Vec::new();
    let mut pos = 0;
    let mut _bit_pos = 0;

    if data.is_empty() {
        return Ok(output);
    }

    // Try stored blocks (BTYPE=00)
    loop {
        if pos >= data.len() { break; }
        let header = data[pos];
        let bfinal = header & 1;
        let btype = (header >> 1) & 3;
        pos += 1;

        if btype == 0 {
            // Stored block
            if pos + 4 > data.len() { break; }
            let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 4; // len + nlen
            if pos + len > data.len() { break; }
            output.extend_from_slice(&data[pos..pos + len]);
            pos += len;
        } else {
            // Dynamic/fixed Huffman - not fully implemented
            return Err("compressed data requires full DEFLATE decoder (not implemented in minimal build)".to_string());
        }

        if bfinal != 0 { break; }
    }

    Ok(output)
}
