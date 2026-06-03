/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! bzcat — decompress bzip2 files to stdout

use bzip2::read::BzDecoder;
use std::fs;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        eprintln!("Usage: bzcat [FILE]...");
        return 0;
    }

    if args.is_empty() {
        return decompress_to_stdout(&mut io::stdin());
    }

    let mut exit_code = 0;
    for file in args {
        let data = match fs::read(file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("bzcat: {file}: {e}");
                exit_code = 1;
                continue;
            }
        };

        if decompress_to_stdout(&mut data.as_slice()) != 0 {
            eprintln!("bzcat: {file}: decompression failed");
            exit_code = 1;
        }
    }
    exit_code
}

fn decompress_to_stdout(input: &mut dyn Read) -> i32 {
    let mut decoder = BzDecoder::new(input);
    let mut buf = [0u8; 8192];
    let stdout = io::stdout();
    let mut out = stdout.lock();
    loop {
        match decoder.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Err(e) = out.write_all(&buf[..n]) {
                    eprintln!("bzcat: write error: {e}");
                    return 1;
                }
            }
            Err(e) => {
                eprintln!("bzcat: decompression error: {e}");
                return 1;
            }
        }
    }
    0
}
