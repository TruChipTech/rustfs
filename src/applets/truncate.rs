/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs::{self, OpenOptions};

pub fn run(args: &[String]) -> i32 {
    let mut size_spec: Option<SizeSpec> = None;
    let mut no_create = false;
    let mut reference: Option<String> = None;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--size" => {
                i += 1;
                if i < args.len() { size_spec = parse_spec(&args[i]); }
            }
            "-c" | "--no-create" => no_create = true,
            "-r" | "--reference" => {
                i += 1;
                if i < args.len() { reference = Some(args[i].clone()); }
            }
            s if s.starts_with("--size=")      => size_spec = parse_spec(&s[7..]),
            s if s.starts_with("--reference=") => reference = Some(s[12..].to_string()),
            s if !s.starts_with('-')            => files.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    if files.is_empty() {
        eprintln!("truncate: missing file operand");
        return 1;
    }

    // Determine effective size
    let spec = if let Some(s) = size_spec {
        s
    } else if let Some(ref rfile) = reference {
        match fs::metadata(rfile) {
            Ok(m) => SizeSpec::Absolute(m.len()),
            Err(e) => { eprintln!("truncate: {rfile}: {e}"); return 1; }
        }
    } else {
        eprintln!("truncate: you must specify either -s SIZE or -r FILE");
        return 1;
    };

    let mut exit_code = 0;
    for file in &files {
        if no_create && !std::path::Path::new(file).exists() {
            continue;
        }
        let result = match OpenOptions::new().write(true).create(!no_create).open(file) {
            Ok(f) => {
                let current = f.metadata().map(|m| m.len()).unwrap_or(0);
                let target = spec.resolve(current);
                f.set_len(target).err()
            }
            Err(e) => Some(e),
        };
        if let Some(e) = result {
            eprintln!("truncate: {file}: {e}");
            exit_code = 1;
        }
    }

    exit_code
}

enum SizeSpec {
    Absolute(u64),
    Increase(u64),
    Decrease(u64),
    RoundDown(u64), // %N
    RoundUp(u64),   // /N
}

impl SizeSpec {
    fn resolve(&self, current: u64) -> u64 {
        match *self {
            SizeSpec::Absolute(n) => n,
            SizeSpec::Increase(n) => current.saturating_add(n),
            SizeSpec::Decrease(n) => current.saturating_sub(n),
            SizeSpec::RoundDown(n) if n > 0 => current / n * n,
            SizeSpec::RoundUp(n)   if n > 0 => current.div_ceil(n) * n,
            _ => current,
        }
    }
}

fn parse_spec(s: &str) -> Option<SizeSpec> {
    if s.is_empty() { return None; }
    let (modifier, rest) = match s.chars().next()? {
        '+' => ('+', &s[1..]),
        '-' => ('-', &s[1..]),
        '<' => ('<', &s[1..]),
        '>' => ('>', &s[1..]),
        '%' => ('%', &s[1..]),
        '/' => ('/', &s[1..]),
        _   => ('=', s),
    };
    let n = parse_size_str(rest)?;
    Some(match modifier {
        '+' => SizeSpec::Increase(n),
        '-' => SizeSpec::Decrease(n),
        '%' => SizeSpec::RoundDown(n),
        '/' => SizeSpec::RoundUp(n),
        _   => SizeSpec::Absolute(n),
    })
}

fn parse_size_str(s: &str) -> Option<u64> {
    if s.is_empty() { return None; }
    let (num, mult): (&str, u64) = if s.ends_with("KB") || s.ends_with("kB") {
        (&s[..s.len()-2], 1000)
    } else if s.ends_with("MB") {
        (&s[..s.len()-2], 1_000_000)
    } else if s.ends_with("GB") {
        (&s[..s.len()-2], 1_000_000_000)
    } else if s.ends_with('K') || s.ends_with('k') {
        (&s[..s.len()-1], 1024)
    } else if s.ends_with('M') || s.ends_with('m') {
        (&s[..s.len()-1], 1024 * 1024)
    } else if s.ends_with('G') || s.ends_with('g') {
        (&s[..s.len()-1], 1024 * 1024 * 1024)
    } else if s.ends_with('T') || s.ends_with('t') {
        (&s[..s.len()-1], 1024u64 * 1024 * 1024 * 1024)
    } else {
        (s, 1)
    };
    num.parse::<u64>().ok().map(|n| n * mult)
}
