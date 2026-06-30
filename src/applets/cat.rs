/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut number_lines = false;
    let mut number_nonblank = false;
    let mut show_ends = false;
    let mut squeeze_blank = false;
    let mut show_tabs = false;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => number_lines = true,
            "-b" => number_nonblank = true,
            "-E" | "-e" => show_ends = true,
            "-s" => squeeze_blank = true,
            "-T" | "-t" => show_tabs = true,
            "-A" => {
                show_ends = true;
                show_tabs = true;
            }
            "--" => {
                files.extend(args[i + 1..].iter().cloned());
                break;
            }
            arg if arg.starts_with('-') && arg.len() > 1 => {
                // Handle combined flags like -nb
                for c in arg[1..].chars() {
                    match c {
                        'n' => number_lines = true,
                        'b' => number_nonblank = true,
                        'E' | 'e' => show_ends = true,
                        's' => squeeze_blank = true,
                        'T' | 't' => show_tabs = true,
                        'A' => {
                            show_ends = true;
                            show_tabs = true;
                        }
                        _ => {
                            eprintln!("cat: unknown option '-{c}'");
                            return 1;
                        }
                    }
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if number_nonblank {
        number_lines = false;
    }

    let sources: Vec<String> = if files.is_empty() {
        vec!["-".to_string()]
    } else {
        files
    };

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut line_num: u64 = 1;
    let mut last_was_blank = false;
    let mut exit_code = 0;

    for source in &sources {
        let reader: Box<dyn Read> = if source == "-" {
            Box::new(io::stdin())
        } else {
            match fs::File::open(source) {
                Ok(f) => Box::new(f),
                Err(e) => {
                    eprintln!("cat: {source}: {e}");
                    exit_code = 1;
                    continue;
                }
            }
        };

        let buf = BufReader::new(reader);

        for line_result in buf.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    // Handle encoding errors gracefully
                    // instead of crashing on invalid UTF-8
                    eprintln!("cat: read error: {e}");
                    exit_code = 1;
                    break;
                }
            };

            let is_blank = line.is_empty();

            if squeeze_blank && is_blank && last_was_blank {
                continue;
            }
            last_was_blank = is_blank;

            let mut display_line = line.clone();
            if show_tabs {
                display_line = display_line.replace('\t', "^I");
            }

            // -b (number non-blank) overrides -n (number all lines).
            let do_number = if number_nonblank { !is_blank } else { number_lines };
            if do_number {
                let _ = write!(out, "{line_num:>6}\t");
                line_num += 1;
            }

            let _ = write!(out, "{display_line}");
            if show_ends {
                let _ = write!(out, "$");
            }
            let _ = writeln!(out);
        }
    }
    let _ = out.flush();
    exit_code
}
