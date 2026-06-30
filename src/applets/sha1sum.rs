/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! sha1sum — compute and check SHA1 message digests
use sha1::{Sha1, Digest};
use std::fs::File;
use std::io::{self, Read};

pub fn run(args: &[String]) -> i32 {
    let mut check = false;
    let mut files = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-c" | "--check" => check = true,
            _ => files.push(arg.clone()),
        }
    }

    if check {
        return check_sums(&files);
    }

    if files.is_empty() {
        files.push("-".to_string());
    }

    let mut exit_code = 0;
    for file in &files {
        match compute(file) {
            Ok(hash) => println!("{hash}  {file}"),
            Err(e) => {
                eprintln!("sha1sum: {file}: {e}");
                exit_code = 1;
            }
        }
    }
    exit_code
}

fn compute(file: &str) -> io::Result<String> {
    let mut hasher = Sha1::new();
    let mut buf = vec![0u8; 8192];
    let mut reader: Box<dyn Read> = if file == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(file)?)
    };
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(e) => return Err(e),
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn check_sums(files: &[String]) -> i32 {
    let mut exit_code = 0;
    for file in files {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("sha1sum: {file}: {e}");
                exit_code = 1;
                continue;
            }
        };
        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(2, "  ").collect();
            if parts.len() != 2 {
                continue;
            }
            match compute(parts[1]) {
                Ok(hash) if hash == parts[0] => println!("{}: OK", parts[1]),
                Ok(_) => {
                    println!("{}: FAILED", parts[1]);
                    exit_code = 1;
                }
                Err(e) => {
                    eprintln!("{}: {e}", parts[1]);
                    exit_code = 1;
                }
            }
        }
    }
    exit_code
}
