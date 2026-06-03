/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("realpath: missing operand");
        return 1;
    }

    let mut exit_code = 0;
    for arg in args {
        match fs::canonicalize(arg) {
            Ok(p) => println!("{}", p.display()),
            Err(e) => {
                eprintln!("realpath: {arg}: {e}");
                exit_code = 1;
            }
        }
    }
    exit_code
}
