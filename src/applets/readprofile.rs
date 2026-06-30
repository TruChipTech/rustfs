/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! readprofile — read kernel profiling information from /proc/profile
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut profile = "/proc/profile".to_string();
    let mut mapfile = "/proc/kallsyms".to_string();
    let mut reset = false;
    let mut info = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => { i += 1; if let Some(p) = args.get(i) { profile = p.clone(); } }
            "-m" => { i += 1; if let Some(m) = args.get(i) { mapfile = m.clone(); } }
            "-r" => reset = true,
            "-i" => info = true,
            _ => {}
        }
        i += 1;
    }

    if reset {
        return match fs::write(&profile, [0u8; 4]) {
            Ok(_) => 0,
            Err(e) => { eprintln!("readprofile: cannot reset {profile}: {e}"); 1 }
        };
    }

    let data = match fs::read(&profile) {
        Ok(d) => d,
        Err(e) => { eprintln!("readprofile: {profile}: {e}"); return 1; }
    };
    if data.len() < std::mem::size_of::<usize>() {
        eprintln!("readprofile: profile buffer too small");
        return 1;
    }

    // First word is the profiling step (sample shift / multiplier).
    let step = usize::from_ne_bytes(data[..std::mem::size_of::<usize>()].try_into().unwrap());
    if info {
        println!("Sampling step: {step}");
        return 0;
    }

    // Resolve text symbol addresses for nicer output (best effort).
    let symbols = load_symbols(&mapfile);
    let wsz = std::mem::size_of::<usize>();
    let mut total = 0usize;
    let mut idx = 1;
    while (idx + 1) * wsz <= data.len() {
        let off = idx * wsz;
        let count = usize::from_ne_bytes(data[off..off + wsz].try_into().unwrap());
        if count != 0 {
            let name = symbols.get(idx).cloned().unwrap_or_else(|| format!("bucket_{idx}"));
            println!("{count:8} {name}");
            total += count;
        }
        idx += 1;
    }
    println!("{total:8} total");
    0
}

fn load_symbols(path: &str) -> Vec<String> {
    // Best-effort: map text symbols in order. Index alignment with profile
    // buckets is approximate; primarily used for human-readable hints.
    let mut out = Vec::new();
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.len() >= 3 && (f[1] == "T" || f[1] == "t") {
                out.push(f[2].to_string());
            }
        }
    }
    out
}
