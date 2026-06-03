/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::env;

pub fn run(args: &[String]) -> i32 {
    let logical = args.iter().any(|a| a == "-L" || a == "--logical");
    let _ = logical; // On most systems, canonicalize gives physical path

    match env::current_dir() {
        Ok(dir) => {
            println!("{}", dir.display());
            0
        }
        Err(e) => {
            eprintln!("pwd: {e}");
            1
        }
    }
}
