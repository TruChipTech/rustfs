/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut recursive = false;
    let mut force = false;
    let mut verbose = false;
    let mut files = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-r" | "-R" | "--recursive" => recursive = true,
            "-f" | "--force" => force = true,
            "-v" | "--verbose" => verbose = true,
            "-rf" | "-fr" => {
                recursive = true;
                force = true;
            }
            _ => files.push(arg.clone()),
        }
    }

    if files.is_empty() {
        eprintln!("rm: missing operand");
        return 1;
    }

    let mut exit_code = 0;

    for file in &files {
        let path = Path::new(file);

        if !path.exists() && !path.is_symlink() {
            if !force {
                eprintln!("rm: cannot remove '{file}': No such file or directory");
                exit_code = 1;
            }
            continue;
        }

        if path.is_dir() {
            if !recursive {
                eprintln!("rm: cannot remove '{file}': Is a directory");
                exit_code = 1;
                continue;
            }

            // Safety: prevent removing "/" or critical paths
            if let Ok(canon) = fs::canonicalize(path) {
                let canon_str = canon.to_string_lossy();
                if canon_str == "/" || canon_str == "\\\\" {
                    eprintln!("rm: refusing to remove '/' or root directory");
                    exit_code = 1;
                    continue;
                }
            }

            match fs::remove_dir_all(path) {
                Ok(()) => {
                    if verbose {
                        println!("removed directory '{file}'");
                    }
                }
                Err(e) => {
                    if !force {
                        eprintln!("rm: cannot remove '{file}': {e}");
                        exit_code = 1;
                    }
                }
            }
        } else {
            match fs::remove_file(path) {
                Ok(()) => {
                    if verbose {
                        println!("removed '{file}'");
                    }
                }
                Err(e) => {
                    if !force {
                        eprintln!("rm: cannot remove '{file}': {e}");
                        exit_code = 1;
                    }
                }
            }
        }
    }

    exit_code
}
