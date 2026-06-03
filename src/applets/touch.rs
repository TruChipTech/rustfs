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
    let mut files = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-c" => {} // change only access time - not commonly used
            _ => files.push(arg.clone()),
        }
    }

    if files.is_empty() {
        eprintln!("touch: missing operand");
        return 1;
    }

    let mut exit_code = 0;

    for file in &files {
        let path = Path::new(file);
        if path.exists() {
            // Update modification time
            // Properly handle read-only files
            match fs::metadata(path) {
                Ok(_meta) => {
                    // Touch the file by opening and writing nothing
                    match fs::OpenOptions::new().write(true).open(path) {
                        Ok(f) => {
                            let _ = f.set_len(f.metadata().map(|m| m.len()).unwrap_or(0));
                        }
                        Err(e) => {
                            eprintln!("touch: cannot touch '{file}': {e}");
                            exit_code = 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("touch: cannot stat '{file}': {e}");
                    exit_code = 1;
                }
            }
        } else {
            // Create the file
            match fs::File::create(path) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("touch: cannot create '{file}': {e}");
                    exit_code = 1;
                }
            }
        }
    }

    exit_code
}
