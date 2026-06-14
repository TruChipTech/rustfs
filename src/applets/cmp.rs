/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::io::{self, Read};

pub fn run(args: &[String]) -> i32 {
    let mut verbose = false; // -l
    let mut silent  = false; // -s
    let mut skip1: u64 = 0;
    let mut skip2: u64 = 0;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-l" | "--verbose"           => verbose = true,
            "-s" | "--quiet" | "--silent" => silent  = true,
            "-i" | "--ignore-initial" => {
                i += 1;
                if i < args.len() {
                    if let Some((a, b)) = args[i].split_once(':') {
                        skip1 = a.parse().unwrap_or(0);
                        skip2 = b.parse().unwrap_or(0);
                    } else {
                        let n: u64 = args[i].parse().unwrap_or(0);
                        skip1 = n; skip2 = n;
                    }
                }
            }
            s if !s.starts_with('-') || s == "-" => files.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    if files.len() < 2 {
        eprintln!("cmp: missing operand");
        eprintln!("Usage: cmp [-ls] [-i N] file1 file2");
        return 1;
    }

    let read_input = |path: &str, skip: u64| -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        if path == "-" {
            io::stdin().read_to_end(&mut buf).map_err(|e| e.to_string())?;
        } else {
            std::fs::File::open(path)
                .and_then(|mut f| f.read_to_end(&mut buf))
                .map_err(|e| format!("{path}: {e}"))?;
        }
        let start = skip.min(buf.len() as u64) as usize;
        Ok(buf[start..].to_vec())
    };

    let data1 = match read_input(&files[0], skip1) {
        Ok(d) => d, Err(e) => { eprintln!("cmp: {e}"); return 2; }
    };
    let data2 = match read_input(&files[1], skip2) {
        Ok(d) => d, Err(e) => { eprintln!("cmp: {e}"); return 2; }
    };

    let maxlen = data1.len().max(data2.len());
    let mut found_diff = false;

    for pos in 0..maxlen {
        let b1 = data1.get(pos).copied();
        let b2 = data2.get(pos).copied();

        match (b1, b2) {
            (None, _) => {
                if !silent {
                    eprintln!("cmp: EOF on {} after byte {pos}", files[0]);
                }
                return 1;
            }
            (_, None) => {
                if !silent {
                    eprintln!("cmp: EOF on {} after byte {pos}", files[1]);
                }
                return 1;
            }
            (Some(a), Some(b)) if a != b => {
                found_diff = true;
                if verbose {
                    println!("{} {:3o} {:3o}", pos + 1, a, b);
                    // keep scanning
                } else if !silent {
                    let line = data1[..pos].iter().filter(|&&x| x == b'\n').count() + 1;
                    println!("{} {} differ: char {}, line {}", files[0], files[1], pos + 1, line);
                    return 1;
                } else {
                    return 1;
                }
            }
            _ => {}
        }
    }

    if found_diff { 1 } else { 0 }
}
