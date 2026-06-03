/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let mut count = false;
    let mut repeated_only = false;
    let mut unique_only = false;
    let mut ignore_case = false;
    let mut skip_fields: usize = 0;
    let mut skip_chars: usize = 0;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--count" => count = true,
            "-d" | "--repeated" => repeated_only = true,
            "-u" | "--unique" => unique_only = true,
            "-i" | "--ignore-case" => ignore_case = true,
            "-f" | "--skip-fields" => {
                i += 1;
                if i < args.len() {
                    skip_fields = args[i].parse().unwrap_or(0);
                }
            }
            "-s" | "--skip-chars" => {
                i += 1;
                if i < args.len() {
                    skip_chars = args[i].parse().unwrap_or(0);
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let lines = super::input_lines(&files);

    if lines.is_empty() {
        return 0;
    }

    let key = |line: &str| -> String {
        let mut s = line.to_string();
        if skip_fields > 0 {
            let parts: Vec<&str> = s.splitn(skip_fields + 1, char::is_whitespace).collect();
            s = parts.last().unwrap_or(&"").to_string();
        }
        if skip_chars > 0 && s.len() > skip_chars {
            s = s[skip_chars..].to_string();
        }
        if ignore_case {
            s.to_lowercase()
        } else {
            s
        }
    };

    let mut groups: Vec<(u64, String)> = Vec::new();
    let mut current = lines[0].clone();
    let mut current_count: u64 = 1;

    for line in &lines[1..] {
        if key(line) == key(&current) {
            current_count += 1;
        } else {
            groups.push((current_count, current.clone()));
            current = line.clone();
            current_count = 1;
        }
    }
    groups.push((current_count, current));

    for (cnt, line) in &groups {
        if repeated_only && *cnt < 2 {
            continue;
        }
        if unique_only && *cnt > 1 {
            continue;
        }

        if count {
            println!("{:>7} {line}", cnt);
        } else {
            println!("{line}");
        }
    }

    0
}
