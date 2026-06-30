/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! run-parts — run all valid executables in a directory, in lexical order.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut test = false;
    let mut list = false;
    let mut extra: Vec<String> = Vec::new();
    let mut dir: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--test" => test = true,
            "--list" => list = true,
            "-a" | "--arg" => {
                if i + 1 < args.len() {
                    extra.push(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("run-parts: option {} requires an argument", args[i]);
                    return 1;
                }
            }
            "--help" => {
                eprintln!("Usage: run-parts [--test] [--list] [-a ARG]... DIRECTORY");
                return 0;
            }
            s if s.starts_with('-') && s != "-" => {
                eprintln!("run-parts: unknown option '{s}'");
                return 1;
            }
            _ => dir = Some(args[i].clone()),
        }
        i += 1;
    }

    let dir = match dir {
        Some(d) => d,
        None => {
            eprintln!("run-parts: missing DIRECTORY");
            return 1;
        }
    };

    let mut names: Vec<String> = match fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| valid_name(n))
            .collect(),
        Err(e) => {
            eprintln!("run-parts: {dir}: {e}");
            return 1;
        }
    };
    names.sort();

    let mut rc = 0;
    for name in names {
        let path = format!("{}/{}", dir.trim_end_matches('/'), name);
        let meta = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.is_file() {
            continue;
        }
        if list {
            println!("{path}");
            continue;
        }
        let executable = meta.permissions().mode() & 0o111 != 0;
        if !executable {
            continue;
        }
        if test {
            println!("{path}");
            continue;
        }
        match Command::new(&path).args(&extra).status() {
            Ok(st) => {
                if !st.success() {
                    let code = st.code().unwrap_or(1);
                    eprintln!("run-parts: {path} exited with code {code}");
                    rc = 1;
                }
            }
            Err(e) => {
                eprintln!("run-parts: failed to run {path}: {e}");
                rc = 1;
            }
        }
    }
    rc
}

/// Valid run-parts names contain only [A-Za-z0-9_-] (no dots/suffixes).
fn valid_name(n: &str) -> bool {
    !n.is_empty()
        && n.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}
