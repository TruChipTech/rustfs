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
    let mut in_place_sfx = String::new();
    let mut quiet = false;
    let mut expressions: Vec<String> = Vec::new();
    let mut script_files: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--quiet" | "--silent"     => quiet = true,
            "-r" | "-E" | "--regexp-extended" => {} // regex crate is already ERE
            "-e" | "--expression" => {
                i += 1;
                if i < args.len() { expressions.push(args[i].clone()); }
            }
            "-f" | "--file" => {
                i += 1;
                if i < args.len() { script_files.push(args[i].clone()); }
            }
            s if s == "-i" || s == "--in-place" => in_place = true,
            s if s.starts_with("-i") && s.len() > 2 => {
                in_place = true;
                in_place_sfx = s[2..].to_string();
            }
            s if s.starts_with("--in-place=") => {
                in_place = true;
                in_place_sfx = s[11..].to_string();
            }
            _ => {
                if expressions.is_empty() && script_files.is_empty() && !args[i].starts_with('-') {
                    expressions.push(args[i].clone());
                } else {
                    files.push(args[i].clone());
                }
            }
        }
        i += 1;
    }

    // Load scripts from -f files
    for sf in &script_files {
        match fs::read_to_string(sf) {
            Ok(content) => {
                for line in content.lines() {
                    let l = line.trim();
                    if !l.is_empty() { expressions.push(l.to_string()); }
                }
            }
            Err(e) => {
                eprintln!("sed: {sf}: {e}");
                return 1;
            }
        }
    }

    if expressions.is_empty() {
        eprintln!("sed: no expression specified");
        return 1;
    }

    let commands: Vec<SedCommand> = expressions
        .iter()
        .filter_map(|e| parse_sed_expr(e))
        .collect();

    if commands.is_empty() {
        eprintln!("sed: invalid expression");
        return 1;
    }

    if files.is_empty() {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut out = stdout.lock();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    let result = apply_commands(&l, &commands);
                    if !quiet { let _ = writeln!(out, "{result}"); }
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
                        if !quiet { output.push_str(&result); output.push('\n'); }
                    }

                    if in_place {
                        if !in_place_sfx.is_empty() {
                            let backup = format!("{file}{in_place_sfx}");
                            if let Err(e) = fs::copy(file, &backup) {
                                eprintln!("sed: {file}: backup failed: {e}");
                                continue;
                            }
                        }
                        let tmp = format!("{file}.rustfs.tmp");
                        match fs::write(&tmp, &output) {
                            Ok(()) => {
                                if let Err(e) = fs::rename(&tmp, file) {
                                    eprintln!("sed: error writing '{file}': {e}");
                                    let _ = fs::remove_file(&tmp);
                                }
                            }
                            Err(e) => eprintln!("sed: error writing '{file}': {e}"),
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
    Substitute { pattern: Regex, replacement: String, global: bool },
    Delete     { pattern: Option<Regex> },
    Print      { pattern: Option<Regex> },
    Append     { pattern: Option<Regex>, text: String },
    Quit,
}

fn parse_sed_expr(expr: &str) -> Option<SedCommand> {
    let expr = expr.trim();

    if expr == "q" || expr == "Q" {
        return Some(SedCommand::Quit);
    }

    if expr.starts_with('s') && expr.len() > 3 {
        let delim = expr.chars().nth(1)?;
        let parts: Vec<&str> = expr[2..].splitn(3, delim).collect();
        if parts.len() < 2 { return None; }
        let (pattern, replacement) = (parts[0], parts[1]);
        let flags = if parts.len() > 2 { parts[2] } else { "" };
        let global = flags.contains('g');
        let case_ins = flags.contains('i') || flags.contains('I');
        let regex_pat = if case_ins { format!("(?i){pattern}") } else { pattern.to_string() };
        let re = Regex::new(&regex_pat).ok()?;
        return Some(SedCommand::Substitute {
            pattern: re,
            replacement: replacement.to_string(),
            global,
        });
    }

    if expr == "d" { return Some(SedCommand::Delete { pattern: None }); }
    if expr == "p" { return Some(SedCommand::Print  { pattern: None }); }

    // /pattern/d  /pattern/p  /pattern/q
    if expr.starts_with('/') {
        let end = expr[1..].find('/')? + 1;
        let inner = &expr[1..end];
        let cmd = expr[end + 1..].trim();
        let re = Regex::new(inner).ok()?;
        return match cmd {
            "d" => Some(SedCommand::Delete { pattern: Some(re) }),
            "p" => Some(SedCommand::Print  { pattern: Some(re) }),
            "q" | "Q" => Some(SedCommand::Quit),
            _ => None,
        };
    }

    // a\TEXT or a TEXT (append text after current line)
    if expr.starts_with('a') {
        let text = expr[1..].trim_start_matches('\\').trim().to_string();
        return Some(SedCommand::Append { pattern: None, text });
    }

    None
}

fn apply_commands(line: &str, commands: &[SedCommand]) -> String {
    let mut result = line.to_string();

    for cmd in commands {
        match cmd {
            SedCommand::Substitute { pattern, replacement, global } => {
                result = if *global {
                    pattern.replace_all(&result, replacement.as_str()).into_owned()
                } else {
                    pattern.replace(&result, replacement.as_str()).into_owned()
                };
            }
            SedCommand::Delete { pattern } => {
                let del = match pattern { Some(re) => re.is_match(&result), None => true };
                if del { return String::new(); }
            }
            SedCommand::Print { pattern } => {
                let do_print = match pattern { Some(re) => re.is_match(&result), None => true };
                if do_print { println!("{result}"); }
            }
            SedCommand::Append { pattern, text } => {
                let do_it = match pattern { Some(re) => re.is_match(&result), None => true };
                if do_it {
                    println!("{result}");
                    println!("{text}");
                    return String::new();
                }
            }
            SedCommand::Quit => {
                println!("{result}");
                std::process::exit(0);
            }
        }
    }

    result
}
