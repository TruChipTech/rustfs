/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

pub fn run(args: &[String]) -> i32 {
    let mut num_lines: usize = 10;
    let mut num_bytes: Option<usize> = None;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                i += 1;
                if i < args.len() {
                    num_lines = args[i].parse().unwrap_or(10);
                }
            }
            "-c" => {
                i += 1;
                if i < args.len() {
                    num_bytes = Some(args[i].parse().unwrap_or(0));
                }
            }
            arg if arg.starts_with("-n") => {
                num_lines = arg[2..].parse().unwrap_or(10);
            }
            arg if arg.starts_with("-c") => {
                num_bytes = Some(arg[2..].parse().unwrap_or(0));
            }
            // Compat: head -5 means head -n 5
            arg if arg.starts_with('-') && arg[1..].parse::<usize>().is_ok() => {
                num_lines = arg[1..].parse().unwrap_or(10);
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if files.is_empty() {
        files.push("-".to_string());
    }

    let show_header = files.len() > 1;
    let mut exit_code = 0;

    for (idx, file) in files.iter().enumerate() {
        if show_header {
            if idx > 0 {
                println!();
            }
            println!("==> {file} <==");
        }

        let reader: Box<dyn Read> = if file == "-" {
            Box::new(io::stdin())
        } else {
            match File::open(file) {
                Ok(f) => Box::new(f),
                Err(e) => {
                    eprintln!("head: cannot open '{file}': {e}");
                    exit_code = 1;
                    continue;
                }
            }
        };

        if let Some(bytes) = num_bytes {
            let mut reader = reader.take(bytes as u64);
            let mut buf = vec![0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        use std::io::Write;
                        let _ = io::stdout().write_all(&buf[..n]);
                    }
                    Err(e) => {
                        eprintln!("head: error reading: {e}");
                        exit_code = 1;
                        break;
                    }
                }
            }
        } else {
            let buf_reader = BufReader::new(reader);
            for (count, line) in buf_reader.lines().enumerate() {
                if count >= num_lines {
                    break;
                }
                match line {
                    Ok(l) => println!("{l}"),
                    Err(e) => {
                        eprintln!("head: error reading: {e}");
                        exit_code = 1;
                        break;
                    }
                }
            }
        }
    }

    exit_code
}
