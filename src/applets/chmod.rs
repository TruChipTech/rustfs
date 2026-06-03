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
    let mut verbose = false;
    let mut sources = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-R" | "-r" | "--recursive" => recursive = true,
            "-v" | "--verbose" => verbose = true,
            _ => sources.push(arg.clone()),
        }
    }

    if sources.len() < 2 {
        eprintln!("chmod: missing operand");
        return 1;
    }

    let mode_str = sources.remove(0);
    let mut exit_code = 0;

    for file in &sources {
        let path = Path::new(file);

        if !path.exists() {
            eprintln!("chmod: cannot access '{file}': No such file or directory");
            exit_code = 1;
            continue;
        }

        if let Err(e) = apply_chmod(path, &mode_str, verbose) {
            eprintln!("chmod: changing permissions of '{file}': {e}");
            exit_code = 1;
        }

        if recursive && path.is_dir() {
            if let Err(e) = chmod_recursive(path, &mode_str, verbose) {
                eprintln!("chmod: error in recursive chmod: {e}");
                exit_code = 1;
            }
        }
    }

    exit_code
}

fn apply_chmod(path: &Path, mode_str: &str, verbose: bool) -> std::io::Result<()> {
    // Try parsing as octal first
    if let Ok(mode) = u32::from_str_radix(mode_str, 8) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(mode);
            fs::set_permissions(path, perms)?;
        }
        #[cfg(not(unix))]
        {
            // On Windows, we can only set readonly
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_readonly(mode & 0o222 == 0);
            fs::set_permissions(path, perms)?;
        }
        if verbose {
            println!("mode of '{}' changed to {:04o}", path.display(), mode);
        }
        return Ok(());
    }

    // Symbolic mode (e.g., u+x, go-w, a+r)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = fs::metadata(path)?;
        let current = meta.permissions().mode();
        let new_mode = parse_symbolic_mode(mode_str, current)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        let perms = fs::Permissions::from_mode(new_mode);
        fs::set_permissions(path, perms)?;
        if verbose {
            println!("mode of '{}' changed to {:04o}", path.display(), new_mode);
        }
    }
    #[cfg(not(unix))]
    {
        // Best effort on Windows
        let _ = verbose;
        let _ = mode_str;
    }

    Ok(())
}

#[cfg(unix)]
fn parse_symbolic_mode(mode_str: &str, current: u32) -> Result<u32, String> {
    let mut mode = current & 0o7777;

    for part in mode_str.split(',') {
        let mut who_mask = 0u32;
        let mut chars = part.chars().peekable();

        // Parse who (u/g/o/a)
        while let Some(&c) = chars.peek() {
            match c {
                'u' => {
                    who_mask |= 0o700;
                    chars.next();
                }
                'g' => {
                    who_mask |= 0o070;
                    chars.next();
                }
                'o' => {
                    who_mask |= 0o007;
                    chars.next();
                }
                'a' => {
                    who_mask |= 0o777;
                    chars.next();
                }
                _ => break,
            }
        }

        if who_mask == 0 {
            who_mask = 0o777; // default to 'a'
        }

        // Parse operator (+, -, =)
        let op = chars.next().ok_or("missing operator")?;
        if op != '+' && op != '-' && op != '=' {
            return Err(format!("invalid operator '{op}'"));
        }

        // Parse permissions
        let mut perm_bits = 0u32;
        for c in chars {
            match c {
                'r' => perm_bits |= 0o444,
                'w' => perm_bits |= 0o222,
                'x' => perm_bits |= 0o111,
                'X' => {
                    // Execute only if directory or already has execute
                    if current & 0o111 != 0 {
                        perm_bits |= 0o111;
                    }
                }
                's' => perm_bits |= 0o4000 | 0o2000,
                't' => perm_bits |= 0o1000,
                _ => return Err(format!("invalid permission '{c}'")),
            }
        }

        let effective = perm_bits & who_mask;

        match op {
            '+' => mode |= effective,
            '-' => mode &= !effective,
            '=' => {
                mode &= !who_mask;
                mode |= effective;
            }
            _ => unreachable!(),
        }
    }

    Ok(mode)
}

fn chmod_recursive(path: &Path, mode_str: &str, verbose: bool) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        apply_chmod(&entry_path, mode_str, verbose)?;
        if entry_path.is_dir() {
            chmod_recursive(&entry_path, mode_str, verbose)?;
        }
    }
    Ok(())
}
