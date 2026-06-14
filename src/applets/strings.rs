/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::io::{self, Read};

pub fn run(args: &[String]) -> i32 {
    let mut min_len: usize = 4;
    let mut offset_fmt: Option<char> = None;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" | "-" => {}
            "-t" | "--radix" => {
                i += 1;
                if i < args.len() { offset_fmt = args[i].chars().next(); }
            }
            "-n" | "--bytes" | "--min-len" => {
                i += 1;
                if i < args.len() { min_len = args[i].parse().unwrap_or(4); }
            }
            s if s.starts_with("--min-len=") => {
                min_len = s[10..].parse().unwrap_or(4);
            }
            s if s.starts_with("-n") && s.len() > 2 => {
                min_len = s[2..].parse().unwrap_or(4);
            }
            s if s.starts_with('-') && s[1..].parse::<usize>().is_ok() => {
                min_len = s[1..].parse().unwrap_or(4);
            }
            s if s.starts_with("--radix=") => {
                offset_fmt = s[8..].chars().next();
            }
            s if !s.starts_with('-') || s == "-" => files.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    if files.is_empty() { files.push("-".to_string()); }

    let mut exit_code = 0;
    for file in &files {
        exit_code |= scan_file(file, min_len, offset_fmt);
    }
    exit_code
}

fn scan_file(file: &str, min_len: usize, offset_fmt: Option<char>) -> i32 {
    let data = if file == "-" {
        let mut buf = Vec::new();
        if io::stdin().read_to_end(&mut buf).is_err() { return 1; }
        buf
    } else {
        match std::fs::read(file) {
            Ok(d) => d,
            Err(e) => { eprintln!("strings: {file}: {e}"); return 1; }
        }
    };

    let mut current = String::new();
    let mut start: usize = 0;

    let flush = |current: &str, start: usize| {
        if current.len() >= min_len {
            match offset_fmt {
                Some('o') => print!("{:o} ", start),
                Some('d') => print!("{} ", start),
                Some('x') => print!("{:x} ", start),
                _ => {}
            }
            println!("{current}");
        }
    };

    for (pos, &byte) in data.iter().enumerate() {
        if (0x20..=0x7e).contains(&byte) || byte == b'\t' {
            if current.is_empty() { start = pos; }
            current.push(byte as char);
        } else {
            flush(&current, start);
            current.clear();
        }
    }
    flush(&current, start);

    0
}
