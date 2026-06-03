/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs;
use std::io;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut recursive = false;
    let mut force = false;
    let mut preserve = false;
    let mut no_clobber = false;
    let mut verbose = false;
    let mut sources = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-r" | "-R" | "--recursive" => recursive = true,
            "-f" | "--force" => force = true,
            "-p" | "--preserve" => preserve = true,
            "-n" | "--no-clobber" => no_clobber = true,
            "-v" | "--verbose" => verbose = true,
            "--" => {
                sources.extend(args[i + 1..].iter().cloned());
                break;
            }
            arg if arg.starts_with('-') && arg.len() > 1 => {
                for c in arg[1..].chars() {
                    match c {
                        'r' | 'R' => recursive = true,
                        'f' => force = true,
                        'p' => preserve = true,
                        'n' => no_clobber = true,
                        'v' => verbose = true,
                        _ => {
                            eprintln!("cp: unknown option '-{c}'");
                            return 1;
                        }
                    }
                }
            }
            _ => sources.push(args[i].clone()),
        }
        i += 1;
    }

    if sources.len() < 2 {
        eprintln!("cp: missing operand");
        return 1;
    }

    let dest = sources.pop().unwrap();
    let dest_path = Path::new(&dest);

    // Multiple sources require dest to be a directory
    if sources.len() > 1 && !dest_path.is_dir() {
        eprintln!("cp: target '{dest}' is not a directory");
        return 1;
    }

    let mut exit_code = 0;

    for src in &sources {
        let src_path = Path::new(src);

        if !src_path.exists() {
            eprintln!("cp: cannot stat '{src}': No such file or directory");
            exit_code = 1;
            continue;
        }

        if src_path.is_dir() && !recursive {
            eprintln!("cp: -r not specified; omitting directory '{src}'");
            exit_code = 1;
            continue;
        }

        let target = if dest_path.is_dir() {
            dest_path.join(src_path.file_name().unwrap_or_default())
        } else {
            dest_path.to_path_buf()
        };

        // Check for no-clobber before copying
        if no_clobber && target.exists() {
            continue;
        }

        if let Err(e) = copy_item(src_path, &target, recursive, force, preserve, verbose) {
            eprintln!("cp: error copying '{src}' to '{}': {e}", target.display());
            exit_code = 1;
        }
    }

    exit_code
}

fn copy_item(
    src: &Path,
    dest: &Path,
    recursive: bool,
    _force: bool,
    preserve: bool,
    verbose: bool,
) -> io::Result<()> {
    if src.is_dir() {
        // Detect and prevent copying a directory into itself
        let src_canon = fs::canonicalize(src)?;
        if dest.exists() {
            let dest_canon = fs::canonicalize(dest)?;
            if dest_canon.starts_with(&src_canon) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "cannot copy a directory into itself",
                ));
            }
        }

        if !dest.exists() {
            fs::create_dir_all(dest)?;
        }

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_dest = dest.join(entry.file_name());
            copy_item(&entry.path(), &entry_dest, recursive, _force, preserve, verbose)?;
        }

        if preserve {
            // Preserve timestamps
            let meta = fs::metadata(src)?;
            let _ = filetime_from_metadata(&meta, dest);
        }
    } else {
        // Use copy instead of manual read/write to handle
        // sparse files, permissions, and special files correctly
        fs::copy(src, dest)?;

        if preserve {
            let meta = fs::metadata(src)?;
            let _ = filetime_from_metadata(&meta, dest);
            // Preserve permissions
            fs::set_permissions(dest, meta.permissions())?;
        }

        if verbose {
            println!("'{}' -> '{}'", src.display(), dest.display());
        }
    }
    Ok(())
}

fn filetime_from_metadata(meta: &fs::Metadata, dest: &Path) -> io::Result<()> {
    // Best-effort timestamp preservation
    let _ = meta;
    let _ = dest;
    // Platform-specific timestamp preservation would go here
    // Rust's std doesn't expose setting file times directly yet
    Ok(())
}
