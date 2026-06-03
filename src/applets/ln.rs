/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs;
use std::path::Path;
#[cfg(unix)]
use std::os::unix::fs as unix_fs;

pub fn run(args: &[String]) -> i32 {
    let mut symbolic = false;
    let mut force = false;
    let mut verbose = false;
    let mut sources = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-s" | "--symbolic" => symbolic = true,
            "-f" | "--force" => force = true,
            "-v" | "--verbose" => verbose = true,
            _ => sources.push(arg.clone()),
        }
    }

    if sources.len() < 2 {
        eprintln!("ln: missing operand");
        return 1;
    }

    let target = sources.pop().unwrap();
    let target_path = Path::new(&target);

    if sources.len() > 1 && !target_path.is_dir() {
        eprintln!("ln: target '{target}' is not a directory");
        return 1;
    }

    let mut exit_code = 0;

    for src in &sources {
        let link_path = if target_path.is_dir() {
            let src_name = Path::new(src)
                .file_name()
                .unwrap_or_default();
            target_path.join(src_name)
        } else {
            target_path.to_path_buf()
        };

        if force && link_path.exists() {
            let _ = fs::remove_file(&link_path);
        }

        let result = if symbolic {
            #[cfg(unix)]
            {
                unix_fs::symlink(src, &link_path)
            }
            #[cfg(not(unix))]
            {
                // On Windows, use junction or symlink depending on target type
                if Path::new(src).is_dir() {
                    std::os::windows::fs::symlink_dir(src, &link_path)
                } else {
                    std::os::windows::fs::symlink_file(src, &link_path)
                }
            }
        } else {
            fs::hard_link(src, &link_path)
        };

        match result {
            Ok(()) => {
                if verbose {
                    println!("'{}' -> '{src}'", link_path.display());
                }
            }
            Err(e) => {
                eprintln!("ln: failed to create link '{}': {e}", link_path.display());
                exit_code = 1;
            }
        }
    }

    exit_code
}
