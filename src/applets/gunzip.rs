/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! gunzip — decompress gzip files

use flate2::read::GzDecoder;
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

        let gz_data = match fs::read(file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("gunzip: {file}: {e}");
                exit_code = 1;
                continue;
            }
        };

        let decompressed = match decompress_bytes(&gz_data) {
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
    let stdin = io::stdin();
    let mut decoder = GzDecoder::new(stdin.lock());
    let mut output = Vec::new();
    match decoder.read_to_end(&mut output) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("gunzip: stdin: {e}");
            return 1;
        }
    }
    if let Err(e) = io::stdout().write_all(&output) {
        eprintln!("gunzip: {e}");
        return 1;
    }
    0
}

fn decompress_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = GzDecoder::new(data);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output).map_err(|e| e.to_string())?;
    Ok(output)
}
