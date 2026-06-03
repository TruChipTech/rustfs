/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

pub fn run(args: &[String]) -> i32 {
    let mut ignore_case = false;
    let mut invert = false;
    let mut count_only = false;
    let mut line_numbers = false;
    let mut files_with_matches = false;
    let mut files_without_matches = false;
    let mut recursive = false;
    let mut fixed_string = false;
    let mut whole_word = false;
    let mut quiet = false;
    let mut max_count: Option<usize> = None;
    let mut after_context: usize = 0;
    let mut before_context: usize = 0;
    let mut pattern = String::new();
    let mut files = Vec::new();
    let mut pattern_set = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-i" | "--ignore-case" => ignore_case = true,
            "-v" | "--invert-match" => invert = true,
            "-c" | "--count" => count_only = true,
            "-n" | "--line-number" => line_numbers = true,
            "-l" | "--files-with-matches" => files_with_matches = true,
            "-L" | "--files-without-match" => files_without_matches = true,
            "-r" | "-R" | "--recursive" => recursive = true,
            "-F" | "--fixed-strings" => fixed_string = true,
            "-w" | "--word-regexp" => whole_word = true,
            "-q" | "--quiet" | "--silent" => quiet = true,
            "-m" | "--max-count" => {
                i += 1;
                if i < args.len() {
                    max_count = args[i].parse().ok();
                }
            }
            "-A" | "--after-context" => {
                i += 1;
                if i < args.len() {
                    after_context = args[i].parse().unwrap_or(0);
                }
            }
            "-B" | "--before-context" => {
                i += 1;
                if i < args.len() {
                    before_context = args[i].parse().unwrap_or(0);
                }
            }
            "-C" | "--context" => {
                i += 1;
                if i < args.len() {
                    let n = args[i].parse().unwrap_or(0);
                    before_context = n;
                    after_context = n;
                }
            }
            "-e" | "--regexp" => {
                i += 1;
                if i < args.len() {
                    pattern = args[i].clone();
                    pattern_set = true;
                }
            }
            arg if arg.starts_with('-') && arg.len() > 1 => {
                // Handle combined short flags
                for c in arg[1..].chars() {
                    match c {
                        'i' => ignore_case = true,
                        'v' => invert = true,
                        'c' => count_only = true,
                        'n' => line_numbers = true,
                        'l' => files_with_matches = true,
                        'L' => files_without_matches = true,
                        'r' | 'R' => recursive = true,
                        'F' => fixed_string = true,
                        'w' => whole_word = true,
                        'q' => quiet = true,
                        _ => {
                            eprintln!("grep: unknown option '-{c}'");
                            return 2;
                        }
                    }
                }
            }
            _ => {
                if !pattern_set {
                    pattern = args[i].clone();
                    pattern_set = true;
                } else {
                    files.push(args[i].clone());
                }
            }
        }
        i += 1;
    }

    if !pattern_set {
        eprintln!("grep: missing pattern");
        return 2;
    }

    // Build regex
    let regex_pattern = if fixed_string {
        regex::escape(&pattern)
    } else if whole_word {
        format!(r"\b{}\b", pattern)
    } else {
        pattern.clone()
    };

    let regex_pattern = if ignore_case {
        format!("(?i){regex_pattern}")
    } else {
        regex_pattern
    };

    let re = match Regex::new(&regex_pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("grep: invalid pattern: {e}");
            return 2;
        }
    };

    // Collect files
    if recursive && files.is_empty() {
        files.push(".".to_string());
    }
    if files.is_empty() {
        files.push("-".to_string());
    }

    // Expand recursive directories
    let mut expanded_files = Vec::new();
    for file in &files {
        if recursive && std::path::Path::new(file).is_dir() {
            for entry in walkdir::WalkDir::new(file)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().is_file() {
                    expanded_files.push(entry.path().display().to_string());
                }
            }
        } else {
            expanded_files.push(file.clone());
        }
    }

    let show_filename = expanded_files.len() > 1;
    let mut found_any = false;

    for file in &expanded_files {
        let reader: Box<dyn Read> = if file == "-" {
            Box::new(io::stdin())
        } else {
            match File::open(file) {
                Ok(f) => Box::new(f),
                Err(e) => {
                    eprintln!("grep: {file}: {e}");
                    continue;
                }
            }
        };

        let buf = BufReader::new(reader);
        let mut count = 0u64;
        let mut file_matched = false;
        let mut lines: Vec<String> = Vec::new();
        let mut match_indices: Vec<usize> = Vec::new();

        for (line_num, line_result) in buf.lines().enumerate() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue, // Skip binary/invalid lines
            };

            let matches = re.is_match(&line);
            let matches = if invert { !matches } else { matches };

            if before_context > 0 || after_context > 0 {
                lines.push(line.clone());
                if matches {
                    match_indices.push(line_num);
                }
            }

            if matches {
                if let Some(max) = max_count {
                    if count >= max as u64 {
                        break;
                    }
                }

                found_any = true;
                file_matched = true;
                count += 1;

                if quiet {
                    return 0;
                }

                if files_with_matches || files_without_matches {
                    continue;
                }

                if !count_only && before_context == 0 && after_context == 0 {
                    if show_filename {
                        print!("{file}:");
                    }
                    if line_numbers {
                        print!("{}:", line_num + 1);
                    }
                    println!("{line}");
                }
            }
        }

        // Handle context output
        if (before_context > 0 || after_context > 0) && !count_only && !files_with_matches {
            let mut printed = std::collections::HashSet::new();
            for &idx in &match_indices {
                let start = idx.saturating_sub(before_context);
                let end = std::cmp::min(idx + after_context + 1, lines.len());

                if !printed.is_empty() && start > *printed.iter().max().unwrap_or(&0) + 1 {
                    println!("--");
                }

                for j in start..end {
                    if printed.contains(&j) {
                        continue;
                    }
                    printed.insert(j);

                    let sep = if j == idx { ':' } else { '-' };
                    if show_filename {
                        print!("{file}{sep}");
                    }
                    if line_numbers {
                        print!("{}{sep}", j + 1);
                    }
                    println!("{}", lines[j]);
                }
            }
        }

        if count_only {
            if show_filename {
                println!("{file}:{count}");
            } else {
                println!("{count}");
            }
        }

        if files_with_matches && file_matched {
            println!("{file}");
        }

        if files_without_matches && !file_matched {
            println!("{file}");
        }
    }

    if found_any { 0 } else { 1 }
}
