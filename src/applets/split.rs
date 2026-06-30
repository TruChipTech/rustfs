/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! split — split a file into pieces
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

enum Mode {
    Lines(usize),
    Bytes(u64),
}

pub fn run(args: &[String]) -> i32 {
    let mut mode = Mode::Lines(1000);
    let mut input = "-".to_string();
    let mut prefix = "x".to_string();
    let mut positional = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(n) = a.strip_prefix("-l") {
            let v = if n.is_empty() { i += 1; args.get(i).cloned().unwrap_or_default() } else { n.to_string() };
            mode = Mode::Lines(v.parse().unwrap_or(1000));
        } else if let Some(n) = a.strip_prefix("-b") {
            let v = if n.is_empty() { i += 1; args.get(i).cloned().unwrap_or_default() } else { n.to_string() };
            mode = Mode::Bytes(parse_size(&v));
        } else if a == "-" || !a.starts_with('-') {
            positional.push(a.clone());
        }
        i += 1;
    }
    if let Some(f) = positional.first() { input = f.clone(); }
    if let Some(p) = positional.get(1) { prefix = p.clone(); }

    let mut reader: Box<dyn BufRead> = if input == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        match File::open(&input) {
            Ok(f) => Box::new(BufReader::new(f)),
            Err(e) => { eprintln!("split: {input}: {e}"); return 1; }
        }
    };

    match mode {
        Mode::Lines(n) => split_lines(&mut reader, &prefix, n.max(1)),
        Mode::Bytes(n) => split_bytes(&mut reader, &prefix, n.max(1)),
    }
}

fn parse_size(s: &str) -> u64 {
    let s = s.trim();
    let (num, mult) = if let Some(p) = s.strip_suffix('k').or_else(|| s.strip_suffix('K')) {
        (p, 1024)
    } else if let Some(p) = s.strip_suffix('m').or_else(|| s.strip_suffix('M')) {
        (p, 1024 * 1024)
    } else if let Some(p) = s.strip_suffix('b') {
        (p, 512)
    } else {
        (s, 1)
    };
    num.parse::<u64>().unwrap_or(1000) * mult
}

fn suffix(idx: usize) -> String {
    let a = (idx / 26) % 26;
    let b = idx % 26;
    format!("{}{}", (b'a' + a as u8) as char, (b'a' + b as u8) as char)
}

fn split_lines(reader: &mut dyn BufRead, prefix: &str, n: usize) -> i32 {
    let mut idx = 0;
    let mut count = 0;
    let mut out: Option<File> = None;
    for line in reader.lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        if out.is_none() || count == n {
            let name = format!("{prefix}{}", suffix(idx));
            out = match File::create(&name) {
                Ok(f) => Some(f),
                Err(e) => { eprintln!("split: {name}: {e}"); return 1; }
            };
            idx += 1;
            count = 0;
        }
        if let Some(f) = out.as_mut() {
            let _ = writeln!(f, "{line}");
        }
        count += 1;
    }
    0
}

fn split_bytes(reader: &mut dyn BufRead, prefix: &str, n: u64) -> i32 {
    let mut idx = 0;
    let mut buf = vec![0u8; 65536];
    let mut remaining = 0u64;
    let mut out: Option<File> = None;
    loop {
        let r = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(r) => r,
            Err(e) => { eprintln!("split: {e}"); return 1; }
        };
        let mut pos = 0;
        while pos < r {
            if out.is_none() || remaining == 0 {
                let name = format!("{prefix}{}", suffix(idx));
                out = match File::create(&name) {
                    Ok(f) => Some(f),
                    Err(e) => { eprintln!("split: {name}: {e}"); return 1; }
                };
                idx += 1;
                remaining = n;
            }
            let take = ((r - pos) as u64).min(remaining) as usize;
            if let Some(f) = out.as_mut() {
                let _ = f.write_all(&buf[pos..pos + take]);
            }
            pos += take;
            remaining -= take as u64;
        }
    }
    0
}
