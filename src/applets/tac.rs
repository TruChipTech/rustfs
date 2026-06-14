/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::io::{self, BufRead};

pub fn run(args: &[String]) -> i32 {
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--separator" => i += 1, // skip separator value (not implemented)
            s if s.starts_with("--separator=") => {}
            s => files.push(s.to_string()),
        }
        i += 1;
    }

    if files.is_empty() {
        files.push("-".to_string());
    }

    let mut exit_code = 0;
    for file in &files {
        exit_code |= tac_file(file);
    }
    exit_code
}

fn tac_file(file: &str) -> i32 {
    let lines: Vec<String> = if file == "-" {
        let stdin = io::stdin();
        stdin.lock().lines().filter_map(|l| l.ok()).collect()
    } else {
        match std::fs::read_to_string(file) {
            Ok(content) => content.lines().map(str::to_string).collect(),
            Err(e) => {
                eprintln!("tac: {file}: {e}");
                return 1;
            }
        }
    };

    for line in lines.iter().rev() {
        println!("{line}");
    }
    0
}
