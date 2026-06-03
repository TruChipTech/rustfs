/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use md5::{Md5, Digest};
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
        match compute_md5(file) {
            Ok(hash) => println!("{hash}  {file}"),
            Err(e) => {
                eprintln!("md5sum: {file}: {e}");
                exit_code = 1;
            }
        }
    }

    exit_code
}

fn compute_md5(file: &str) -> io::Result<String> {
    let mut hasher = Md5::new();
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

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn check_sums(files: &[String]) -> i32 {
    let mut exit_code = 0;

    for file in files {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("md5sum: {file}: {e}");
                exit_code = 1;
                continue;
            }
        };

        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(2, "  ").collect();
            if parts.len() != 2 {
                continue;
            }
            let expected = parts[0];
            let check_file = parts[1];

            match compute_md5(check_file) {
                Ok(hash) => {
                    if hash == expected {
                        println!("{check_file}: OK");
                    } else {
                        println!("{check_file}: FAILED");
                        exit_code = 1;
                    }
                }
                Err(e) => {
                    eprintln!("{check_file}: {e}");
                    exit_code = 1;
                }
            }
        }
    }

    exit_code
}
