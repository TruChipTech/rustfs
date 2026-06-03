/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use regex::Regex;
use std::fs;
use std::io::{self, BufRead, Write};

pub fn run(args: &[String]) -> i32 {
    let mut in_place = false;
    let mut quiet = false;
    let mut expressions = Vec::new();
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-i" | "--in-place" => in_place = true,
            "-n" | "--quiet" | "--silent" => quiet = true,
            "-e" | "--expression" => {
                i += 1;
                if i < args.len() {
                    expressions.push(args[i].clone());
                }
            }
            _ => {
                if expressions.is_empty() && !args[i].starts_with('-') {
                    expressions.push(args[i].clone());
                } else {
                    files.push(args[i].clone());
                }
            }
        }
        i += 1;
    }

    if expressions.is_empty() {
        eprintln!("sed: no expression specified");
        return 1;
    }

    // Parse sed expressions
    let commands: Vec<SedCommand> = expressions
        .iter()
        .filter_map(|e| parse_sed_expr(e))
        .collect();

    if commands.is_empty() {
        eprintln!("sed: invalid expression");
        return 1;
    }

    if files.is_empty() {
        // Read from stdin
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut out = stdout.lock();

        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    let result = apply_commands(&l, &commands);
                    if !quiet {
                        let _ = writeln!(out, "{result}");
                    }
                }
                Err(_) => break,
            }
        }
    } else {
        for file in &files {
            match fs::read_to_string(file) {
                Ok(content) => {
                    let mut output = String::new();
                    for line in content.lines() {
                        let result = apply_commands(line, &commands);
                        if !quiet {
                            output.push_str(&result);
                            output.push('\n');
                        }
                    }

                    if in_place {
                        // Safety: write to temp file first, then rename
                        // to prevent data loss on write failure
                        let tmp = format!("{file}.rustfs.tmp");
                        match fs::write(&tmp, &output) {
                            Ok(()) => {
                                if let Err(e) = fs::rename(&tmp, file) {
                                    eprintln!("sed: error writing '{file}': {e}");
                                    let _ = fs::remove_file(&tmp);
                                }
                            }
                            Err(e) => {
                                eprintln!("sed: error writing '{file}': {e}");
                            }
                        }
                    } else {
                        print!("{output}");
                    }
                }
                Err(e) => {
                    eprintln!("sed: {file}: {e}");
                    return 1;
                }
            }
        }
    }

    0
}

enum SedCommand {
    Substitute {
        pattern: Regex,
        replacement: String,
        global: bool,
    },
    Delete {
        pattern: Option<Regex>,
    },
    Print {
        pattern: Option<Regex>,
    },
}

fn parse_sed_expr(expr: &str) -> Option<SedCommand> {
    let expr = expr.trim();

    if expr.starts_with('s') && expr.len() > 3 {
        let delim = expr.chars().nth(1)?;
        let parts: Vec<&str> = expr[2..].splitn(3, delim).collect();
        if parts.len() < 2 {
            return None;
        }
        let pattern = parts[0];
        let replacement = parts[1];
        let flags = if parts.len() > 2 { parts[2] } else { "" };

        let global = flags.contains('g');
        let case_insensitive = flags.contains('i') || flags.contains('I');

        let regex_pat = if case_insensitive {
            format!("(?i){pattern}")
        } else {
            pattern.to_string()
        };

        let re = Regex::new(&regex_pat).ok()?;
        Some(SedCommand::Substitute {
            pattern: re,
            replacement: replacement.to_string(),
            global,
        })
    } else if expr == "d" || expr.ends_with("/d") {
        if expr == "d" {
            Some(SedCommand::Delete { pattern: None })
        } else {
            // /pattern/d
            let inner = &expr[1..expr.len() - 2];
            let re = Regex::new(inner).ok()?;
            Some(SedCommand::Delete { pattern: Some(re) })
        }
    } else if expr == "p" || expr.ends_with("/p") {
        if expr == "p" {
            Some(SedCommand::Print { pattern: None })
        } else {
            let inner = &expr[1..expr.len() - 2];
            let re = Regex::new(inner).ok()?;
            Some(SedCommand::Print { pattern: Some(re) })
        }
    } else {
        None
    }
}

fn apply_commands(line: &str, commands: &[SedCommand]) -> String {
    let mut result = line.to_string();

    for cmd in commands {
        match cmd {
            SedCommand::Substitute {
                pattern,
                replacement,
                global,
            } => {
                if *global {
                    result = pattern.replace_all(&result, replacement.as_str()).to_string();
                } else {
                    result = pattern.replace(&result, replacement.as_str()).to_string();
                }
            }
            SedCommand::Delete { pattern } => {
                let should_delete = match pattern {
                    Some(re) => re.is_match(&result),
                    None => true,
                };
                if should_delete {
                    return String::new();
                }
            }
            SedCommand::Print { pattern } => {
                let should_print = match pattern {
                    Some(re) => re.is_match(&result),
                    None => true,
                };
                if should_print {
                    println!("{result}");
                }
            }
        }
    }

    result
}
