/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let mut reverse = false;
    let mut numeric = false;
    let mut unique = false;
    let mut stable = false;
    let mut ignore_case = false;
    let mut check = false;
    let mut output_file: Option<String> = None;
    let mut key_field: Option<usize> = None;
    let mut separator = None;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-r" | "--reverse" => reverse = true,
            "-n" | "--numeric-sort" => numeric = true,
            "-u" | "--unique" => unique = true,
            "-s" | "--stable" => stable = true,
            "-f" | "--ignore-case" => ignore_case = true,
            "-c" | "--check" => check = true,
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_file = Some(args[i].clone());
                }
            }
            "-k" | "--key" => {
                i += 1;
                if i < args.len() {
                    // Parse simple key spec (e.g., "2" or "2,2")
                    let key_spec = &args[i];
                    key_field = key_spec.split(',').next()
                        .and_then(|k| k.parse::<usize>().ok());
                }
            }
            "-t" => {
                i += 1;
                if i < args.len() {
                    separator = args[i].chars().next();
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let mut lines = super::input_lines(&files);

    if check {
        // Check if input is sorted
        for i in 1..lines.len() {
            let cmp = compare_lines(&lines[i - 1], &lines[i], numeric, ignore_case, key_field, separator);
            if cmp == std::cmp::Ordering::Greater {
                eprintln!("sort: disorder: {}", lines[i]);
                return 1;
            }
        }
        return 0;
    }

    // Use stable sort to preserve relative order of equal elements
    if stable || true {
        // Always use stable sort
        lines.sort_by(|a, b| {
            compare_lines(a, b, numeric, ignore_case, key_field, separator)
        });
    }

    if reverse {
        lines.reverse();
    }

    if unique {
        lines.dedup();
    }

    let output: String = lines.join("\n");
    if let Some(out_file) = output_file {
        match std::fs::write(&out_file, format!("{output}\n")) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("sort: write error: {e}");
                return 1;
            }
        }
    } else {
        for line in &lines {
            println!("{line}");
        }
    }

    0
}

fn compare_lines(
    a: &str,
    b: &str,
    numeric: bool,
    ignore_case: bool,
    key_field: Option<usize>,
    separator: Option<char>,
) -> std::cmp::Ordering {
    let (a_key, b_key) = if let Some(field) = key_field {
        let sep = separator.unwrap_or(' ');
        let a_parts: Vec<&str> = a.split(sep).collect();
        let b_parts: Vec<&str> = b.split(sep).collect();
        let idx = field.saturating_sub(1);
        (
            a_parts.get(idx).unwrap_or(&"").to_string(),
            b_parts.get(idx).unwrap_or(&"").to_string(),
        )
    } else {
        (a.to_string(), b.to_string())
    };

    if numeric {
        let a_num: f64 = a_key.trim().parse().unwrap_or(0.0);
        let b_num: f64 = b_key.trim().parse().unwrap_or(0.0);
        a_num.partial_cmp(&b_num).unwrap_or(std::cmp::Ordering::Equal)
    } else if ignore_case {
        a_key.to_lowercase().cmp(&b_key.to_lowercase())
    } else {
        a_key.cmp(&b_key)
    }
}
