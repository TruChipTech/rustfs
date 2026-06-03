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
    let mut force = false;
    let mut no_clobber = false;
    let mut verbose = false;
    let mut sources = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-f" | "--force" => force = true,
            "-n" | "--no-clobber" => no_clobber = true,
            "-v" | "--verbose" => verbose = true,
            _ => sources.push(arg.clone()),
        }
    }

    if sources.len() < 2 {
        eprintln!("mv: missing operand");
        return 1;
    }

    let dest = sources.pop().unwrap();
    let dest_path = Path::new(&dest);

    if sources.len() > 1 && !dest_path.is_dir() {
        eprintln!("mv: target '{dest}' is not a directory");
        return 1;
    }

    let mut exit_code = 0;

    for src in &sources {
        let src_path = Path::new(src);

        if !src_path.exists() {
            eprintln!("mv: cannot stat '{src}': No such file or directory");
            exit_code = 1;
            continue;
        }

        let target = if dest_path.is_dir() {
            dest_path.join(src_path.file_name().unwrap_or_default())
        } else {
            dest_path.to_path_buf()
        };

        // Properly handle no-clobber
        if no_clobber && target.exists() {
            continue;
        }

        // Check if source and destination are the same file
        if src_path.exists() && target.exists() {
            if let (Ok(src_canon), Ok(dst_canon)) =
                (fs::canonicalize(src_path), fs::canonicalize(&target))
            {
                if src_canon == dst_canon {
                    eprintln!("mv: '{src}' and '{}' are the same file", target.display());
                    exit_code = 1;
                    continue;
                }
            }
        }

        if !force && target.exists() && target.metadata().map(|m| m.permissions().readonly()).unwrap_or(false) {
            eprintln!("mv: cannot overwrite '{}': Permission denied", target.display());
            exit_code = 1;
            continue;
        }

        // Try rename first (same filesystem), fall back to copy+delete
        match fs::rename(src_path, &target) {
            Ok(()) => {
                if verbose {
                    println!("renamed '{src}' -> '{}'", target.display());
                }
            }
            Err(_) => {
                // Cross-filesystem move: copy then delete
                if src_path.is_dir() {
                    match copy_dir_recursive(src_path, &target) {
                        Ok(()) => {
                            let _ = fs::remove_dir_all(src_path);
                            if verbose {
                                println!("'{src}' -> '{}'", target.display());
                            }
                        }
                        Err(e) => {
                            eprintln!("mv: cannot move '{src}': {e}");
                            exit_code = 1;
                        }
                    }
                } else {
                    match fs::copy(src_path, &target) {
                        Ok(_) => {
                            let _ = fs::remove_file(src_path);
                            if verbose {
                                println!("'{src}' -> '{}'", target.display());
                            }
                        }
                        Err(e) => {
                            eprintln!("mv: cannot move '{src}': {e}");
                            exit_code = 1;
                        }
                    }
                }
            }
        }
    }

    exit_code
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
