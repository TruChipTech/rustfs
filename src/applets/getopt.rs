/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! getopt — parse command options

use std::io::Write;

pub fn run(args: &[String]) -> i32 {
    // getopt optstring parameters
    // or: getopt -o optstring [-l longopts] -- parameters
    let mut optstring = String::new();
    let mut longopts: Vec<String> = Vec::new();
    let mut params: Vec<String> = Vec::new();
    let mut name = "getopt".to_string();

    let mut i = 0;
    let mut found_separator = false;

    while i < args.len() {
        if found_separator {
            params.push(args[i].clone());
        } else {
            match args[i].as_str() {
                "-o" | "--options" => {
                    i += 1;
                    if i < args.len() { optstring = args[i].clone(); }
                }
                "-l" | "--longoptions" => {
                    i += 1;
                    if i < args.len() {
                        longopts = args[i].split(',').map(|s| s.to_string()).collect();
                    }
                }
                "-n" | "--name" => {
                    i += 1;
                    if i < args.len() { name = args[i].clone(); }
                }
                "--" => found_separator = true,
                "-h" | "--help" => {
                    eprintln!("Usage: getopt -o OPTSTRING [-l LONGOPTS] [-n NAME] -- PARAMS...");
                    return 0;
                }
                s => {
                    if optstring.is_empty() && !s.starts_with('-') {
                        optstring = s.to_string();
                    } else {
                        params.push(s.to_string());
                    }
                }
            }
        }
        i += 1;
    }

    if optstring.is_empty() && params.is_empty() {
        eprintln!("Usage: getopt optstring parameters");
        return 1;
    }

    // If no separator was found, the params start after optstring
    if !found_separator && params.is_empty() && args.len() > 1 {
        // Traditional mode: first arg is optstring, rest are params
        if !args.is_empty() {
            optstring = args[0].clone();
            params = args[1..].to_vec();
        }
    }

    // Parse the parameters
    let mut output = Vec::new();
    let mut non_options = Vec::new();
    let mut pi = 0;

    while pi < params.len() {
        let param = &params[pi];
        if param == "--" {
            non_options.extend_from_slice(&params[pi + 1..]);
            break;
        }
        if param.starts_with("--") {
            // Long option
            let opt_name = param.trim_start_matches('-');
            let (key, value) = if let Some(pos) = opt_name.find('=') {
                (&opt_name[..pos], Some(&opt_name[pos + 1..]))
            } else {
                (opt_name, None)
            };

            let found = longopts.iter().any(|l| {
                l.trim_end_matches(':') == key
            });
            if found {
                output.push(format!("--{key}"));
                let needs_arg = longopts.iter().any(|l| l.ends_with(':') && l.trim_end_matches(':') == key);
                if needs_arg {
                    if let Some(v) = value {
                        output.push(format!("'{v}'"));
                    } else {
                        pi += 1;
                        if pi < params.len() {
                            output.push(format!("'{}'", params[pi]));
                        }
                    }
                }
            } else {
                eprintln!("{name}: unrecognized option '--{key}'");
                return 1;
            }
        } else if param.starts_with('-') && param.len() > 1 {
            // Short options
            let chars: Vec<char> = param[1..].chars().collect();
            let mut ci = 0;
            while ci < chars.len() {
                let c = chars[ci];
                let pos = optstring.find(c);
                if let Some(p) = pos {
                    output.push(format!("-{c}"));
                    // Check if it takes an argument
                    if optstring.get(p + 1..p + 2) == Some(":") {
                        if ci + 1 < chars.len() {
                            let rest: String = chars[ci + 1..].iter().collect();
                            output.push(format!("'{rest}'"));
                            break;
                        } else {
                            pi += 1;
                            if pi < params.len() {
                                output.push(format!("'{}'", params[pi]));
                            }
                        }
                    }
                } else {
                    eprintln!("{name}: invalid option -- '{c}'");
                    return 1;
                }
                ci += 1;
            }
        } else {
            non_options.push(param.clone());
        }
        pi += 1;
    }

    output.push("--".to_string());
    for no in &non_options {
        output.push(format!("'{no}'"));
    }

    let _ = std::io::stdout().write_all(format!("{}\n", output.join(" ")).as_bytes());
    0
}
