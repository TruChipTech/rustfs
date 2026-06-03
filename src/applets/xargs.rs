/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::io::{self, BufRead};
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut max_args: Option<usize> = None;
    let mut delimiter = '\n';
    let mut no_run_if_empty = false;
    let mut verbose = false;
    let mut command = vec!["echo".to_string()];
    let mut use_null = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--max-args" => {
                i += 1;
                if i < args.len() {
                    max_args = args[i].parse().ok();
                }
            }
            "-d" | "--delimiter" => {
                i += 1;
                if i < args.len() {
                    delimiter = args[i].chars().next().unwrap_or('\n');
                }
            }
            "-0" | "--null" => {
                use_null = true;
                delimiter = '\0';
            }
            "-r" | "--no-run-if-empty" => no_run_if_empty = true,
            "-t" | "--verbose" => verbose = true,
            "--" => {
                command = args[i + 1..].to_vec();
                break;
            }
            _ => {
                command = args[i..].to_vec();
                break;
            }
        }
        i += 1;
    }

    // Read input
    let stdin = io::stdin();
    let mut items = Vec::new();

    if use_null {
        let mut buf = String::new();
        let _ = stdin.lock().read_to_string(&mut buf);
        items = buf.split('\0').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
    } else {
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    if delimiter == '\n' {
                        let trimmed = l.trim().to_string();
                        if !trimmed.is_empty() {
                            items.push(trimmed);
                        }
                    } else {
                        for part in l.split(delimiter) {
                            let trimmed = part.trim().to_string();
                            if !trimmed.is_empty() {
                                items.push(trimmed);
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }

    if items.is_empty() && no_run_if_empty {
        return 0;
    }

    let batch_size = max_args.unwrap_or(items.len());
    let mut exit_code = 0;

    for chunk in items.chunks(batch_size.max(1)) {
        let cmd = &command[0];
        let mut cmd_args: Vec<&str> = command[1..].iter().map(|s| s.as_str()).collect();
        let chunk_strs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
        cmd_args.extend(chunk_strs);

        if verbose {
            eprintln!("{} {}", cmd, cmd_args.join(" "));
        }

        match Command::new(cmd).args(&cmd_args).status() {
            Ok(status) => {
                if !status.success() {
                    exit_code = status.code().unwrap_or(1);
                }
            }
            Err(e) => {
                eprintln!("xargs: {cmd}: {e}");
                exit_code = 127;
            }
        }
    }

    exit_code
}

use std::io::Read;
