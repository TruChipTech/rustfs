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
    let mut count_lines = false;
    let mut count_words = false;
    let mut count_chars = false;
    let mut count_bytes = false;
    let mut files = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-l" => count_lines = true,
            "-w" => count_words = true,
            "-m" => count_chars = true,
            "-c" => count_bytes = true,
            _ => files.push(arg.clone()),
        }
    }

    // Default: show all
    if !count_lines && !count_words && !count_chars && !count_bytes {
        count_lines = true;
        count_words = true;
        count_bytes = true;
    }

    if files.is_empty() {
        files.push("-".to_string());
    }

    let mut total_lines: u64 = 0;
    let mut total_words: u64 = 0;
    let mut total_chars: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut exit_code = 0;
    let show_total = files.len() > 1;

    for file in &files {
        let reader: Box<dyn Read> = if file == "-" {
            Box::new(io::stdin())
        } else {
            match File::open(file) {
                Ok(f) => Box::new(f),
                Err(e) => {
                    eprintln!("wc: {file}: {e}");
                    exit_code = 1;
                    continue;
                }
            }
        };

        let buf = BufReader::new(reader);
        let mut lines: u64 = 0;
        let mut words: u64 = 0;
        let mut chars: u64 = 0;
        let mut bytes: u64 = 0;

        for line_result in buf.lines() {
            match line_result {
                Ok(line) => {
                    lines += 1;
                    words += line.split_whitespace().count() as u64;
                    chars += line.chars().count() as u64 + 1; // +1 for newline
                    bytes += line.len() as u64 + 1;
                }
                Err(_) => break,
            }
        }

        total_lines += lines;
        total_words += words;
        total_chars += chars;
        total_bytes += bytes;

        print_counts(
            lines, words, chars, bytes, file, count_lines, count_words, count_chars, count_bytes,
        );
    }

    if show_total {
        print_counts(
            total_lines,
            total_words,
            total_chars,
            total_bytes,
            "total",
            count_lines,
            count_words,
            count_chars,
            count_bytes,
        );
    }

    exit_code
}

fn print_counts(
    lines: u64,
    words: u64,
    chars: u64,
    bytes: u64,
    name: &str,
    show_lines: bool,
    show_words: bool,
    show_chars: bool,
    show_bytes: bool,
) {
    let mut parts = Vec::new();
    if show_lines {
        parts.push(format!("{:>7}", lines));
    }
    if show_words {
        parts.push(format!("{:>7}", words));
    }
    if show_chars {
        parts.push(format!("{:>7}", chars));
    }
    if show_bytes {
        parts.push(format!("{:>7}", bytes));
    }
    println!("{} {name}", parts.join(""));
}
