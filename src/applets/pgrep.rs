/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! pgrep — look up processes by name (pattern)
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let (matches, list) = collect(args, "pgrep");
    if list {
        for (pid, name) in &matches {
            println!("{pid} {name}");
        }
    } else {
        for (pid, _) in &matches {
            println!("{pid}");
        }
    }
    if matches.is_empty() { 1 } else { 0 }
}

/// Shared scan used by both pgrep and pkill. Returns (pid, comm) pairs and
/// whether the -l (list name) flag was given.
pub fn collect(args: &[String], _prog: &str) -> (Vec<(i32, String)>, bool) {
    let mut list = false;
    let mut full = false;
    let mut exact = false;
    let mut invert = false;
    let mut pattern = String::new();

    for a in args {
        match a.as_str() {
            "-l" => list = true,
            "-f" => full = true,
            "-x" => exact = true,
            "-v" => invert = true,
            s if s.starts_with('-') && s.len() > 1 => {} // ignore unknown single flags
            s => pattern = s.to_string(),
        }
    }

    let re = regex::Regex::new(&pattern).ok();
    let me = std::process::id() as i32;
    let mut out = Vec::new();
    let dir = match fs::read_dir("/proc") { Ok(d) => d, Err(_) => return (out, list) };
    for entry in dir.flatten() {
        let dname = entry.file_name().to_string_lossy().to_string();
        let pid: i32 = match dname.parse() { Ok(p) => p, Err(_) => continue };
        if pid == me { continue; }
        let comm = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
        let comm = comm.trim().to_string();
        let haystack = if full {
            fs::read_to_string(format!("/proc/{pid}/cmdline"))
                .unwrap_or_default()
                .replace('\0', " ")
                .trim()
                .to_string()
        } else {
            comm.clone()
        };
        let m = if exact {
            haystack == pattern
        } else if let Some(re) = &re {
            re.is_match(&haystack)
        } else {
            false
        };
        if m != invert {
            out.push((pid, comm));
        }
    }
    out.sort_by_key(|(p, _)| *p);
    (out, list)
}
