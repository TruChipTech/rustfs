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
    let mut whole_line = false; // -x
    let mut only_matching = false; // -o
    let mut quiet = false;
    let mut max_count: Option<usize> = None;
    let mut after_context: usize = 0;
    let mut before_context: usize = 0;
    // -H/-h override the automatic filename logic
    let mut force_filename: Option<bool> = None; // Some(true)=-H, Some(false)=-h
    let mut patterns: Vec<String> = Vec::new();
    let mut pattern_files: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    let mut pattern_set = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-i" | "--ignore-case"          => ignore_case = true,
            "-v" | "--invert-match"         => invert = true,
            "-c" | "--count"                => count_only = true,
            "-n" | "--line-number"          => line_numbers = true,
            "-l" | "--files-with-matches"   => files_with_matches = true,
            "-L" | "--files-without-match"  => files_without_matches = true,
            "-r" | "-R" | "--recursive"     => recursive = true,
            "-F" | "--fixed-strings"        => fixed_string = true,
            "-w" | "--word-regexp"          => whole_word = true,
            "-x" | "--line-regexp"          => whole_line = true,
            "-o" | "--only-matching"        => only_matching = true,
            "-q" | "--quiet" | "--silent"   => quiet = true,
            "-H" | "--with-filename"        => force_filename = Some(true),
            "-h" | "--no-filename"          => force_filename = Some(false),
            "-E" | "--extended-regexp"      => {} // regex crate is already ERE
            "-P" | "--perl-regexp"          => {} // close enough
            "-G" | "--basic-regexp"         => {} // accepted
            "-m" | "--max-count" => {
                i += 1;
                if i < args.len() { max_count = args[i].parse().ok(); }
            }
            "-A" | "--after-context" => {
                i += 1;
                if i < args.len() { after_context = args[i].parse().unwrap_or(0); }
            }
            "-B" | "--before-context" => {
                i += 1;
                if i < args.len() { before_context = args[i].parse().unwrap_or(0); }
            }
            "-C" | "--context" => {
                i += 1;
                if i < args.len() {
                    let n = args[i].parse().unwrap_or(0);
                    before_context = n; after_context = n;
                }
            }
            // Attached-number forms: -A2, -B3, -C1
            a if a.len() > 2 && a.starts_with("-A") && a[2..].chars().all(|c| c.is_ascii_digit()) => {
                after_context = a[2..].parse().unwrap_or(0);
            }
            a if a.len() > 2 && a.starts_with("-B") && a[2..].chars().all(|c| c.is_ascii_digit()) => {
                before_context = a[2..].parse().unwrap_or(0);
            }
            a if a.len() > 2 && a.starts_with("-C") && a[2..].chars().all(|c| c.is_ascii_digit()) => {
                let n = a[2..].parse().unwrap_or(0);
                before_context = n; after_context = n;
            }
            "-e" | "--regexp" => {
                i += 1;
                if i < args.len() { patterns.push(args[i].clone()); pattern_set = true; }
            }
            "-f" | "--file" => {
                i += 1;
                if i < args.len() { pattern_files.push(args[i].clone()); pattern_set = true; }
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Combined short flags: -rni, -Hi, etc.
                let mut skip_next = false;
                let chars: Vec<char> = arg[1..].chars().collect();
                let mut j = 0;
                while j < chars.len() {
                    match chars[j] {
                        'i' => ignore_case = true,
                        'v' => invert = true,
                        'c' => count_only = true,
                        'n' => line_numbers = true,
                        'l' => files_with_matches = true,
                        'L' => files_without_matches = true,
                        'r' | 'R' => recursive = true,
                        'F' => fixed_string = true,
                        'w' => whole_word = true,
                        'x' => whole_line = true,
                        'o' => only_matching = true,
                        'q' => quiet = true,
                        'H' => force_filename = Some(true),
                        'h' => force_filename = Some(false),
                        'E' | 'P' | 'G' => {}
                        'm' | 'A' | 'B' | 'C' | 'e' | 'f' => {
                            // These need an argument; parse as separate flags above
                            skip_next = true;
                            break;
                        }
                        _ => {}
                    }
                    j += 1;
                }
                let _ = skip_next; // single-letter combos with args require separate -X VAL form
            }
            _ => {
                if !pattern_set {
                    patterns.push(args[i].clone());
                    pattern_set = true;
                } else {
                    files.push(args[i].clone());
                }
            }
        }
        i += 1;
    }

    // Load patterns from -f files
    for pfile in &pattern_files {
        match std::fs::read_to_string(pfile) {
            Ok(content) => {
                for line in content.lines() {
                    if !line.is_empty() {
                        patterns.push(line.to_string());
                    }
                }
            }
            Err(e) => {
                eprintln!("grep: {pfile}: {e}");
                return 2;
            }
        }
    }

    if patterns.is_empty() {
        eprintln!("grep: missing pattern");
        return 2;
    }

    // Build combined regex from all patterns
    let combined_raw = if fixed_string {
        patterns.iter().map(|p| regex::escape(p)).collect::<Vec<_>>().join("|")
    } else {
        patterns.join("|")
    };

    let combined = if whole_line {
        format!("^(?:{combined_raw})$")
    } else if whole_word {
        format!(r"\b(?:{combined_raw})\b")
    } else {
        combined_raw
    };

    let regex_pattern = if ignore_case { format!("(?i){combined}") } else { combined };

    let re = match Regex::new(&regex_pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("grep: invalid pattern: {e}");
            return 2;
        }
    };

    if recursive && files.is_empty() { files.push(".".to_string()); }
    if files.is_empty() { files.push("-".to_string()); }

    // Expand directories for -r
    let mut expanded_files: Vec<String> = Vec::new();
    for file in &files {
        if recursive && std::path::Path::new(file).is_dir() {
            for entry in walkdir::WalkDir::new(file).follow_links(false).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file() {
                    expanded_files.push(entry.path().display().to_string());
                }
            }
        } else {
            expanded_files.push(file.clone());
        }
    }

    let auto_filename = expanded_files.len() > 1;
    let show_filename = force_filename.unwrap_or(auto_filename);
    let mut found_any = false;

    for file in &expanded_files {
        let reader: Box<dyn Read> = if file == "-" {
            Box::new(io::stdin())
        } else {
            match File::open(file) {
                Ok(f) => Box::new(f),
                Err(e) => { eprintln!("grep: {file}: {e}"); continue; }
            }
        };

        let buf = BufReader::new(reader);
        let mut count = 0u64;
        let mut file_matched = false;
        let mut ctx_lines: Vec<String> = Vec::new();
        let mut match_indices: Vec<usize> = Vec::new();

        for (line_num, line_result) in buf.lines().enumerate() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue,
            };

            let is_match = re.is_match(&line);
            let matched = if invert { !is_match } else { is_match };

            if before_context > 0 || after_context > 0 {
                ctx_lines.push(line.clone());
                if matched { match_indices.push(line_num); }
            }

            if matched {
                if let Some(max) = max_count {
                    if count >= max as u64 { break; }
                }
                found_any = true;
                file_matched = true;
                count += 1;

                if quiet { return 0; }
                if files_with_matches || files_without_matches { continue; }

                if !count_only && before_context == 0 && after_context == 0 {
                    if only_matching && !invert {
                        for mat in re.find_iter(&line) {
                            if show_filename { print!("{file}:"); }
                            if line_numbers  { print!("{}:", line_num + 1); }
                            println!("{}", mat.as_str());
                        }
                    } else {
                        if show_filename { print!("{file}:"); }
                        if line_numbers  { print!("{}:", line_num + 1); }
                        println!("{line}");
                    }
                }
            }
        }

        // Context output
        if (before_context > 0 || after_context > 0) && !count_only && !files_with_matches {
            let mut printed = std::collections::HashSet::new();
            for &idx in &match_indices {
                let start = idx.saturating_sub(before_context);
                let end = std::cmp::min(idx + after_context + 1, ctx_lines.len());
                if !printed.is_empty() && start > *printed.iter().max().unwrap_or(&0) + 1 {
                    println!("--");
                }
                for (offset, line) in ctx_lines[start..end].iter().enumerate() {
                    let j = start + offset;
                    if printed.contains(&j) { continue; }
                    printed.insert(j);
                    let sep = if j == idx { ':' } else { '-' };
                    if show_filename { print!("{file}{sep}"); }
                    if line_numbers  { print!("{}{sep}", j + 1); }
                    println!("{line}");
                }
            }
        }

        if count_only {
            if show_filename { println!("{file}:{count}"); }
            else             { println!("{count}"); }
        }
        if files_with_matches    && file_matched  { println!("{file}"); }
        if files_without_matches && !file_matched { println!("{file}"); }
    }

    if found_any { 0 } else { 1 }
}
