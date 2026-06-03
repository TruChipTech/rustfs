/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ar — create, modify, and extract from archives

use std::fs;
use std::io::Write;

const AR_MAGIC: &[u8] = b"!<arch>\n";

pub fn run(args: &[String]) -> i32 {
    if args.len() < 2 {
        eprintln!("Usage: ar [rcs|t|x] archive [files...]");
        return 1;
    }

    let operation = &args[0];
    let archive = &args[1];
    let files = &args[2..];

    match operation.as_str() {
        s if s.contains('t') => list_archive(archive),
        s if s.contains('x') => extract_archive(archive, files),
        s if s.contains('r') => {
            let create = s.contains('c');
            update_archive(archive, files, create)
        }
        _ => {
            eprintln!("ar: unsupported operation: {operation}");
            1
        }
    }
}

fn list_archive(archive: &str) -> i32 {
    let data = match fs::read(archive) {
        Ok(d) => d,
        Err(e) => { eprintln!("ar: {archive}: {e}"); return 1; }
    };

    if !data.starts_with(AR_MAGIC) {
        eprintln!("ar: {archive}: not an archive");
        return 1;
    }

    let mut offset = AR_MAGIC.len();
    while offset + 60 <= data.len() {
        let header = &data[offset..offset + 60];
        let name = std::str::from_utf8(&header[0..16]).unwrap_or("").trim_end_matches(|c: char| c == ' ' || c == '/');
        let size_str = std::str::from_utf8(&header[48..58]).unwrap_or("").trim();
        let size: usize = size_str.parse().unwrap_or(0);

        if !name.is_empty() {
            println!("{name}");
        }

        offset += 60 + size;
        if offset % 2 != 0 { offset += 1; }
    }
    0
}

fn extract_archive(archive: &str, files: &[String]) -> i32 {
    let data = match fs::read(archive) {
        Ok(d) => d,
        Err(e) => { eprintln!("ar: {archive}: {e}"); return 1; }
    };

    if !data.starts_with(AR_MAGIC) {
        eprintln!("ar: {archive}: not an archive");
        return 1;
    }

    let mut offset = AR_MAGIC.len();
    let mut exit_code = 0;

    while offset + 60 <= data.len() {
        let header = &data[offset..offset + 60];
        let name = std::str::from_utf8(&header[0..16]).unwrap_or("").trim_end_matches(|c: char| c == ' ' || c == '/');
        let size_str = std::str::from_utf8(&header[48..58]).unwrap_or("").trim();
        let size: usize = size_str.parse().unwrap_or(0);

        let should_extract = files.is_empty() || files.iter().any(|f| f == name);

        if should_extract && !name.is_empty() && offset + 60 + size <= data.len() {
            let content = &data[offset + 60..offset + 60 + size];
            if let Err(e) = fs::write(name, content) {
                eprintln!("ar: failed to extract {name}: {e}");
                exit_code = 1;
            }
        }

        offset += 60 + size;
        if offset % 2 != 0 { offset += 1; }
    }
    exit_code
}

fn update_archive(archive: &str, files: &[String], create: bool) -> i32 {
    if files.is_empty() {
        eprintln!("ar: no files to add");
        return 1;
    }

    let mut output: Vec<u8> = Vec::new();

    // Read existing archive or start fresh
    if !create && std::path::Path::new(archive).exists() {
        if let Ok(data) = fs::read(archive) {
            if data.starts_with(AR_MAGIC) {
                output.extend_from_slice(&data);
            }
        }
    }

    if output.is_empty() {
        output.extend_from_slice(AR_MAGIC);
    }

    for file in files {
        let content = match fs::read(file) {
            Ok(c) => c,
            Err(e) => { eprintln!("ar: {file}: {e}"); return 1; }
        };

        let name = std::path::Path::new(file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file);

        // Build ar header (60 bytes)
        let mut header = [b' '; 60];
        let name_field = format!("{name}/");
        let name_bytes = name_field.as_bytes();
        let copy_len = name_bytes.len().min(16);
        header[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

        // Timestamp
        let ts = b"0";
        header[16..16 + ts.len()].copy_from_slice(ts);
        // UID
        header[28..29].copy_from_slice(b"0");
        // GID
        header[34..35].copy_from_slice(b"0");
        // Mode
        let mode = b"100644";
        header[40..40 + mode.len()].copy_from_slice(mode);
        // Size
        let size_str = format!("{}", content.len());
        let size_bytes = size_str.as_bytes();
        header[48..48 + size_bytes.len()].copy_from_slice(size_bytes);
        // Magic
        header[58] = b'`';
        header[59] = b'\n';

        output.extend_from_slice(&header);
        output.extend_from_slice(&content);

        // Pad to even boundary
        if content.len() % 2 != 0 {
            output.push(b'\n');
        }
    }

    match fs::File::create(archive) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(&output) {
                eprintln!("ar: failed to write {archive}: {e}");
                return 1;
            }
        }
        Err(e) => { eprintln!("ar: cannot create {archive}: {e}"); return 1; }
    }
    0
}
