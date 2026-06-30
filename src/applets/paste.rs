/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn run(args: &[String]) -> i32 {
    let mut delimiter = String::from("\t");
    let mut serial = false;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-d" || arg == "--delimiters" {
            i += 1;
            if i < args.len() {
                delimiter = args[i].clone();
            }
        } else if let Some(d) = arg.strip_prefix("-d") {
            delimiter = d.to_string();
        } else if arg == "-s" || arg == "--serial" {
            serial = true;
        } else if arg == "--help" {
            println!("Usage: paste [-d DELIM] [-s] [FILE]...");
            return 0;
        } else if arg == "-" || !arg.starts_with('-') {
            files.push(arg.clone());
        } else {
            // Handle combined flags like -sd,
            let flags = &arg[1..];
            let mut j = 0;
            let chars: Vec<char> = flags.chars().collect();
            while j < chars.len() {
                match chars[j] {
                    's' => serial = true,
                    'd' => {
                        delimiter = chars[j + 1..].iter().collect();
                        break;
                    }
                    _ => {
                        eprintln!("paste: invalid option -- '{}'", chars[j]);
                        return 1;
                    }
                }
                j += 1;
            }
        }
        i += 1;
    }

    if files.is_empty() {
        files.push("-".to_string());
    }

    let file_lines: Vec<Vec<String>> = files
        .iter()
        .map(|f| {
            if f == "-" {
                let stdin = io::stdin();
                BufReader::new(stdin.lock())
                    .lines()
                    .map_while(Result::ok)
                    .collect()
            } else {
                match File::open(f) {
                    Ok(file) => BufReader::new(file)
                        .lines()
                        .map_while(Result::ok)
                        .collect(),
                    Err(e) => {
                        eprintln!("paste: {f}: {e}");
                        Vec::new()
                    }
                }
            }
        })
        .collect();

    if serial {
        for lines in &file_lines {
            println!("{}", lines.join(&delimiter));
        }
    } else {
        let max_lines = file_lines.iter().map(|f| f.len()).max().unwrap_or(0);
        for line_num in 0..max_lines {
            let parts: Vec<&str> = file_lines
                .iter()
                .map(|f| f.get(line_num).map(|s| s.as_str()).unwrap_or(""))
                .collect();
            println!("{}", parts.join(&delimiter));
        }
    }

    0
}
