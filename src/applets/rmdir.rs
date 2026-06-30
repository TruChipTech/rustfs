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
    let mut dirs = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-p" | "--parents" => parents = true,
            _ => dirs.push(arg.clone()),
        }
    }

    if dirs.is_empty() {
        eprintln!("rmdir: missing operand");
        return 1;
    }

    let mut exit_code = 0;

    for dir in &dirs {
        if parents {
            // Remove directory and its empty parents
            let mut path = std::path::PathBuf::from(dir);
            while fs::remove_dir(&path).is_ok() {
                if !path.pop() || path.as_os_str().is_empty() {
                    break;
                }
            }
        } else {
            match fs::remove_dir(dir) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("rmdir: failed to remove '{dir}': {e}");
                    exit_code = 1;
                }
            }
        }
    }

    exit_code
}
