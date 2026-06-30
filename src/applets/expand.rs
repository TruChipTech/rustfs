/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! expand — convert tabs to spaces
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut tabstop = 8usize;
    let mut files = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(n) = a.strip_prefix("-t") {
            if n.is_empty() {
                i += 1;
                tabstop = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(8);
            } else {
                tabstop = n.parse().unwrap_or(8);
            }
        } else if let Some(n) = a.strip_prefix("--tabs=") {
            tabstop = n.parse().unwrap_or(8);
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
                Err(e) => { eprintln!("expand: {file}: {e}"); exit_code = 1; continue; }
            }
        };
        for line in reader.lines() {
            let line = match line { Ok(l) => l, Err(_) => break };
            let mut col = 0;
            let mut s = String::new();
            for c in line.chars() {
                if c == '\t' {
                    let spaces = tabstop - (col % tabstop);
                    for _ in 0..spaces { s.push(' '); }
                    col += spaces;
                } else {
                    s.push(c);
                    col += 1;
                }
            }
            let _ = writeln!(out, "{s}");
        }
    }
    exit_code
}
