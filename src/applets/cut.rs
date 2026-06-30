/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let mut delimiter = '\t';
    let mut fields: Vec<(usize, Option<usize>)> = Vec::new(); // (start, end) 1-indexed
    let mut bytes_mode = false;
    let mut chars_mode = false;
    let mut complement = false;
    let mut only_delimited = false;
    let mut file_args = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--delimiter" => {
                i += 1;
                if i < args.len() {
                    delimiter = args[i].chars().next().unwrap_or('\t');
                }
            }
            "-f" | "--fields" => {
                i += 1;
                if i < args.len() {
                    fields = parse_ranges(&args[i]);
                }
            }
            "-b" | "--bytes" => {
                bytes_mode = true;
                i += 1;
                if i < args.len() {
                    fields = parse_ranges(&args[i]);
                }
            }
            "-c" | "--characters" => {
                chars_mode = true;
                i += 1;
                if i < args.len() {
                    fields = parse_ranges(&args[i]);
                }
            }
            "--complement" => complement = true,
            "-s" | "--only-delimited" => only_delimited = true,
            arg if arg.starts_with("-d") => {
                delimiter = arg[2..].chars().next().unwrap_or('\t');
            }
            arg if arg.starts_with("-f") => {
                fields = parse_ranges(&arg[2..]);
            }
            _ => file_args.push(args[i].clone()),
        }
        i += 1;
    }

    if fields.is_empty() {
        eprintln!("cut: you must specify a list of fields/bytes/characters");
        return 1;
    }

    let lines = super::input_lines(&file_args);

    for line in &lines {
        if bytes_mode || chars_mode {
            let chars: Vec<char> = line.chars().collect();
            let mut selected: Vec<bool> = vec![false; chars.len()];

            for &(start, end) in &fields {
                let s = start.saturating_sub(1);
                let e = end.unwrap_or(start).min(chars.len());
                if s < e {
                    selected[s..e].iter_mut().for_each(|sel| *sel = true);
                }
            }

            if complement {
                for s in &mut selected {
                    *s = !*s;
                }
            }

            let result: String = chars
                .iter()
                .enumerate()
                .filter(|(i, _)| selected.get(*i).copied().unwrap_or(false))
                .map(|(_, c)| c)
                .collect();
            println!("{result}");
        } else {
            // Field mode
            let parts: Vec<&str> = line.split(delimiter).collect();

            if only_delimited && parts.len() == 1 {
                continue;
            }

            let mut selected_indices: Vec<bool> = vec![false; parts.len()];
            for &(start, end) in &fields {
                let s = start.saturating_sub(1);
                let e = end.unwrap_or(start).min(parts.len());
                if s < e {
                    selected_indices[s..e].iter_mut().for_each(|sel| *sel = true);
                }
            }

            if complement {
                for s in &mut selected_indices {
                    *s = !*s;
                }
            }

            let result: Vec<&str> = parts
                .iter()
                .enumerate()
                .filter(|(i, _)| selected_indices.get(*i).copied().unwrap_or(false))
                .map(|(_, s)| *s)
                .collect();
            println!("{}", result.join(&delimiter.to_string()));
        }
    }

    0
}

fn parse_ranges(spec: &str) -> Vec<(usize, Option<usize>)> {
    let mut ranges = Vec::new();
    for part in spec.split(',') {
        if part.contains('-') {
            let parts: Vec<&str> = part.splitn(2, '-').collect();
            let start: usize = parts[0].parse().unwrap_or(1);
            let end: Option<usize> = if parts[1].is_empty() {
                None // open-ended range
            } else {
                parts[1].parse().ok()
            };
            ranges.push((start, end));
        } else if let Ok(n) = part.parse::<usize>() {
            ranges.push((n, Some(n)));
        }
    }
    ranges
}
