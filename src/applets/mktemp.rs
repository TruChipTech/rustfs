/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs::{self, OpenOptions};
use std::io;
use std::path::PathBuf;

pub fn run(args: &[String]) -> i32 {
    let mut dir_mode = false;
    let mut dry_run = false;
    let mut base_dir: Option<String> = None;
    let mut template: Option<String> = None;
    let mut suffix = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--directory" => dir_mode = true,
            "-u" | "--dry-run"   => dry_run = true,
            "-t"                 => {} // deprecated, ignored
            "-p" | "--tmpdir" => {
                i += 1;
                if i < args.len() {
                    base_dir = Some(args[i].clone());
                }
            }
            "-q" | "--quiet" => {}
            s if s.starts_with("--tmpdir=") => {
                base_dir = Some(s[9..].to_string());
            }
            s if s.starts_with("--suffix=") => {
                suffix = s[9..].to_string();
            }
            "--suffix" => {
                i += 1;
                if i < args.len() {
                    suffix = args[i].clone();
                }
            }
            s if !s.starts_with('-') => template = Some(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    let tmpl = template.unwrap_or_else(|| "tmp.XXXXXXXXXX".to_string());

    // Count trailing Xs that will be replaced
    let x_count = tmpl.chars().rev().take_while(|&c| c == 'X').count();
    if x_count < 3 {
        eprintln!("mktemp: too few X's in template '{tmpl}'");
        return 1;
    }

    let prefix = &tmpl[..tmpl.len() - x_count];
    let dir = base_dir
        .or_else(|| std::env::var("TMPDIR").ok())
        .unwrap_or_else(|| "/tmp".to_string());

    match make_unique(&dir, prefix, &suffix, x_count, dir_mode, dry_run) {
        Ok(path) => {
            println!("{}", path.display());
            0
        }
        Err(e) => {
            eprintln!("mktemp: {e}");
            1
        }
    }
}

fn make_unique(
    dir: &str,
    prefix: &str,
    suffix: &str,
    x_count: usize,
    is_dir: bool,
    dry_run: bool,
) -> Result<PathBuf, io::Error> {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let pid = std::process::id() as u64;
    let time_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);

    for attempt in 0u64..256 {
        let mut n = pid
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add(time_ns)
            .wrapping_add(attempt.wrapping_mul(6364136223846793005));

        let rand_part: String = (0..x_count).map(|_| {
            let idx = (n % CHARS.len() as u64) as usize;
            n = n.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            CHARS[idx] as char
        }).collect();

        let name = format!("{prefix}{rand_part}{suffix}");
        let path = PathBuf::from(dir).join(&name);

        if dry_run {
            return Ok(path);
        }

        if is_dir {
            match fs::create_dir(&path) {
                Ok(()) => return Ok(path),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        } else {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(path),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
    }

    Err(io::Error::new(io::ErrorKind::AlreadyExists,
        format!("could not create temp file in '{dir}'")))
}
