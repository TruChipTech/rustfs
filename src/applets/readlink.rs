/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut canonicalize = false;
    let mut files = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-f" | "--canonicalize" => canonicalize = true,
            _ => files.push(arg.clone()),
        }
    }

    if files.is_empty() {
        eprintln!("readlink: missing operand");
        return 1;
    }

    let mut exit_code = 0;

    for file in &files {
        if canonicalize {
            match fs::canonicalize(file) {
                Ok(p) => println!("{}", p.display()),
                Err(e) => {
                    eprintln!("readlink: {file}: {e}");
                    exit_code = 1;
                }
            }
        } else {
            match fs::read_link(file) {
                Ok(target) => println!("{}", target.display()),
                Err(e) => {
                    eprintln!("readlink: {file}: {e}");
                    exit_code = 1;
                }
            }
        }
    }

    exit_code
}
