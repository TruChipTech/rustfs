/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use walkdir::WalkDir;

pub fn run(args: &[String]) -> i32 {
    let mut paths = Vec::new();
    let mut name_pattern: Option<String> = None;
    let mut file_type: Option<char> = None;
    let mut max_depth: Option<usize> = None;
    let mut min_depth: Option<usize> = None;
    let mut print0 = false;
    let mut exec_cmd: Option<Vec<String>> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-name" => {
                i += 1;
                if i < args.len() {
                    name_pattern = Some(args[i].clone());
                }
            }
            "-type" => {
                i += 1;
                if i < args.len() {
                    file_type = args[i].chars().next();
                }
            }
            "-maxdepth" => {
                i += 1;
                if i < args.len() {
                    max_depth = args[i].parse().ok();
                }
            }
            "-mindepth" => {
                i += 1;
                if i < args.len() {
                    min_depth = args[i].parse().ok();
                }
            }
            "-print0" => print0 = true,
            "-exec" => {
                i += 1;
                let mut cmd = Vec::new();
                while i < args.len() && args[i] != ";" {
                    cmd.push(args[i].clone());
                    i += 1;
                }
                exec_cmd = Some(cmd);
            }
            arg if !arg.starts_with('-') && paths.is_empty() => {
                paths.push(arg.to_string());
            }
            arg if !arg.starts_with('-') => {
                paths.push(arg.to_string());
            }
            _ => {} // ignore unknown options
        }
        i += 1;
    }

    if paths.is_empty() {
        paths.push(".".to_string());
    }

    let mut exit_code = 0;

    for search_path in &paths {
        let mut walker = WalkDir::new(search_path).follow_links(false);

        if let Some(max) = max_depth {
            walker = walker.max_depth(max);
        }
        if let Some(min) = min_depth {
            walker = walker.min_depth(min);
        }

        for entry in walker.into_iter() {
            match entry {
                Ok(e) => {
                    let path = e.path();

                    // Type filter
                    if let Some(ft) = file_type {
                        let matches = match ft {
                            'f' => path.is_file(),
                            'd' => path.is_dir(),
                            'l' => path.symlink_metadata()
                                .map(|m| m.file_type().is_symlink())
                                .unwrap_or(false),
                            _ => true,
                        };
                        if !matches {
                            continue;
                        }
                    }

                    // Name filter (glob-style)
                    if let Some(ref pattern) = name_pattern {
                        let name = path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        if !glob_match(pattern, &name) {
                            continue;
                        }
                    }

                    // Execute command or print
                    if let Some(ref cmd) = exec_cmd {
                        if !cmd.is_empty() {
                            let path_str = path.display().to_string();
                            let actual_args: Vec<String> = cmd[1..]
                                .iter()
                                .map(|a| a.replace("{}", &path_str))
                                .collect();

                            match std::process::Command::new(&cmd[0])
                                .args(&actual_args)
                                .status()
                            {
                                Ok(status) => {
                                    if !status.success() {
                                        exit_code = 1;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("find: exec error: {e}");
                                    exit_code = 1;
                                }
                            }
                        }
                    } else if print0 {
                        print!("{}\0", path.display());
                    } else {
                        println!("{}", path.display());
                    }
                }
                Err(e) => {
                    eprintln!("find: {e}");
                    exit_code = 1;
                }
            }
        }
    }

    exit_code
}

/// Simple glob matching supporting * and ?
fn glob_match(pattern: &str, name: &str) -> bool {
    let pat_chars: Vec<char> = pattern.chars().collect();
    let name_chars: Vec<char> = name.chars().collect();
    glob_match_inner(&pat_chars, &name_chars)
}

fn glob_match_inner(pattern: &[char], name: &[char]) -> bool {
    let mut pi = 0;
    let mut ni = 0;
    let mut star_pi = None;
    let mut star_ni = None;

    while ni < name.len() {
        if pi < pattern.len() && (pattern[pi] == '?' || pattern[pi] == name[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < pattern.len() && pattern[pi] == '*' {
            star_pi = Some(pi);
            star_ni = Some(ni);
            pi += 1;
        } else if let Some(spi) = star_pi {
            pi = spi + 1;
            let sni = star_ni.unwrap() + 1;
            star_ni = Some(sni);
            ni = sni;
        } else {
            return false;
        }
    }

    while pi < pattern.len() && pattern[pi] == '*' {
        pi += 1;
    }

    pi == pattern.len()
}
