/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut parents = false;
    let mut verbose = false;
    let mut dirs = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-p" | "--parents" => parents = true,
            "-v" | "--verbose" => verbose = true,
            "-m" => {
                // mode flag - skip for now, mkdir creates with umask
            }
            _ => dirs.push(arg.clone()),
        }
    }

    if dirs.is_empty() {
        eprintln!("mkdir: missing operand");
        return 1;
    }

    let mut exit_code = 0;

    for dir in &dirs {
        let result = if parents {
            fs::create_dir_all(dir)
        } else {
            fs::create_dir(dir)
        };

        match result {
            Ok(()) => {
                if verbose {
                    println!("mkdir: created directory '{dir}'");
                }
            }
            Err(e) => {
                // Don't error on existing directory with -p
                if parents && std::path::Path::new(dir).is_dir() {
                    continue;
                }
                eprintln!("mkdir: cannot create directory '{dir}': {e}");
                exit_code = 1;
            }
        }
    }

    exit_code
}
