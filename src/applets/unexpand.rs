/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! unexpand — convert leading spaces to tabs
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut tabstop = 8usize;
    let mut all = false;
    let mut files = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "-a" || a == "--all" {
            all = true;
        } else if let Some(n) = a.strip_prefix("-t") {
            if n.is_empty() {
                i += 1;
                tabstop = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(8);
            } else {
                tabstop = n.parse().unwrap_or(8);
            }
            all = true;
        } else if let Some(n) = a.strip_prefix("--tabs=") {
            tabstop = n.parse().unwrap_or(8);
            all = true;
        } else if a == "-" || !a.starts_with('-') {
            files.push(a.clone());
        }
        i += 1;
    }
    if tabstop == 0 { tabstop = 1; }
    if files.is_empty() {
        files.push("-".to_string());
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut exit_code = 0;
    for file in &files {
        let reader: Box<dyn BufRead> = if file == "-" {
            Box::new(BufReader::new(io::stdin()))
        } else {
            match File::open(file) {
                Ok(f) => Box::new(BufReader::new(f)),
                Err(e) => { eprintln!("unexpand: {file}: {e}"); exit_code = 1; continue; }
            }
        };
        for line in reader.lines() {
            let line = match line { Ok(l) => l, Err(_) => break };
            let _ = writeln!(out, "{}", convert(&line, tabstop, all));
        }
    }
    exit_code
}

fn convert(line: &str, tabstop: usize, all: bool) -> String {
    let mut out = String::new();
    let mut col = 0usize;
    let mut spaces = 0usize;
    let mut spaces_start = 0usize;
    let mut leading = true;
    for c in line.chars() {
        if c == ' ' && (all || leading) {
            if spaces == 0 { spaces_start = col; }
            spaces += 1;
            col += 1;
            if (spaces_start + spaces).is_multiple_of(tabstop) {
                out.push('\t');
                spaces = 0;
                spaces_start = col;
            }
        } else {
            for _ in 0..spaces { out.push(' '); }
            spaces = 0;
            out.push(c);
            col += 1;
            if c != ' ' { leading = false; }
            spaces_start = col;
        }
    }
    for _ in 0..spaces { out.push(' '); }
    out
}
