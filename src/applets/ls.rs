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
use std::os::unix::fs::PermissionsExt;

pub fn run(args: &[String]) -> i32 {
    let mut show_all = false;
    let mut long_format = false;
    let mut human_readable = false;
    let mut one_per_line = false;
    let mut show_hidden = false;
    let mut reverse = false;
    let mut sort_by_time = false;
    let mut sort_by_size = false;
    let mut recursive = false;
    let mut dirs = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => show_all = true,
            "-A" => show_hidden = true,
            "-l" => long_format = true,
            "-h" | "--human-readable" => human_readable = true,
            "-1" => one_per_line = true,
            "-r" | "--reverse" => reverse = true,
            "-t" => sort_by_time = true,
            "-S" => sort_by_size = true,
            "-R" | "--recursive" => recursive = true,
            "--" => {
                dirs.extend(args[i + 1..].iter().cloned());
                break;
            }
            arg if arg.starts_with('-') && arg.len() > 1 => {
                for c in arg[1..].chars() {
                    match c {
                        'a' => show_all = true,
                        'A' => show_hidden = true,
                        'l' => long_format = true,
                        'h' => human_readable = true,
                        '1' => one_per_line = true,
                        'r' => reverse = true,
                        't' => sort_by_time = true,
                        'S' => sort_by_size = true,
                        'R' => recursive = true,
                        _ => {
                            eprintln!("ls: unknown option '-{c}'");
                            return 1;
                        }
                    }
                }
            }
            _ => dirs.push(args[i].clone()),
        }
        i += 1;
    }

    if dirs.is_empty() {
        dirs.push(".".to_string());
    }

    let show_dir_header = dirs.len() > 1 || recursive;
    let mut exit_code = 0;

    for (idx, dir) in dirs.iter().enumerate() {
        if idx > 0 {
            println!();
        }
        if show_dir_header {
            println!("{dir}:");
        }
        if let Err(e) = list_dir(
            dir,
            show_all || show_hidden,
            long_format,
            human_readable,
            one_per_line,
            reverse,
            sort_by_time,
            sort_by_size,
            recursive,
            show_dir_header,
            show_all,
        ) {
            eprintln!("ls: cannot access '{dir}': {e}");
            exit_code = 1;
        }
    }

    exit_code
}

fn list_dir(
    dir: &str,
    show_hidden: bool,
    long_format: bool,
    human_readable: bool,
    one_per_line: bool,
    reverse: bool,
    sort_by_time: bool,
    sort_by_size: bool,
    recursive: bool,
    show_dir_header: bool,
    show_dot_dirs: bool,
) -> std::io::Result<()> {
    let path = Path::new(dir);

    // If it's a file, just show it
    if path.is_file() {
        if long_format {
            print_long_entry(path, human_readable);
        } else {
            println!("{}", path.file_name().unwrap_or_default().to_string_lossy());
        }
        return Ok(());
    }

    let mut entries: Vec<fs::DirEntry> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .collect();

    // Sort entries
    if sort_by_time {
        entries.sort_by(|a, b| {
            let ta = a.metadata().and_then(|m| m.modified()).ok();
            let tb = b.metadata().and_then(|m| m.modified()).ok();
            tb.cmp(&ta)
        });
    } else if sort_by_size {
        entries.sort_by(|a, b| {
            let sa = a.metadata().map(|m| m.len()).unwrap_or(0);
            let sb = b.metadata().map(|m| m.len()).unwrap_or(0);
            sb.cmp(&sa)
        });
    } else {
        entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    }

    if reverse {
        entries.reverse();
    }

    // Filter hidden files
    let entries: Vec<&fs::DirEntry> = entries
        .iter()
        .filter(|e| {
            if show_hidden {
                true
            } else {
                let name = e.file_name().to_string_lossy().to_string();
                !name.starts_with('.')
            }
        })
        .collect();

    if long_format {
        // Print total block size
        let total_size: u64 = entries
            .iter()
            .filter_map(|e| e.metadata().ok())
            .map(|m| (m.len() + 511) / 512)
            .sum();
        println!("total {total_size}");

        for entry in &entries {
            print_long_entry(&entry.path(), human_readable);
        }
    } else if one_per_line {
        for entry in &entries {
            println!("{}", entry.file_name().to_string_lossy());
        }
    } else {
        // Simple column output
        let names: Vec<String> = entries
            .iter()
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    format!("{name}/")
                } else {
                    name
                }
            })
            .collect();
        print_columns(&names);
    }

    // Recursive listing
    if recursive {
        for entry in &entries {
            if entry.path().is_dir() {
                let subdir = entry.path().to_string_lossy().to_string();
                println!("\n{subdir}:");
                let _ = list_dir(
                    &subdir,
                    show_hidden,
                    long_format,
                    human_readable,
                    one_per_line,
                    reverse,
                    sort_by_time,
                    sort_by_size,
                    recursive,
                    show_dir_header,
                    show_dot_dirs,
                );
            }
        }
    }

    Ok(())
}

fn print_long_entry(path: &Path, human_readable: bool) {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return,
    };

    let file_type = if meta.is_dir() {
        "d"
    } else if meta.file_type().is_symlink() {
        "l"
    } else {
        "-"
    };

    let size = meta.len();
    let size_str = if human_readable {
        human_size(size)
    } else {
        format!("{:>8}", size)
    };

    #[cfg(unix)]
    let perms = format_permissions_unix(meta.permissions().mode());
    #[cfg(not(unix))]
    let perms = if meta.permissions().readonly() {
        "r--r--r--"
    } else {
        "rw-rw-rw-"
    }
    .to_string();

    let modified = meta
        .modified()
        .ok()
        .and_then(|t| {
            let datetime: chrono::DateTime<chrono::Local> = t.into();
            Some(datetime.format("%b %e %H:%M").to_string())
        })
        .unwrap_or_else(|| "???".to_string());

    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    // Check for symlink target
    let display_name = if meta.file_type().is_symlink() {
        if let Ok(target) = fs::read_link(path) {
            format!("{name} -> {}", target.display())
        } else {
            name.to_string()
        }
    } else {
        name.to_string()
    };

    println!(
        "{file_type}{perms} {size_str} {modified} {display_name}"
    );
}

#[cfg(unix)]
fn format_permissions_unix(mode: u32) -> String {
    let mut s = String::with_capacity(9);
    let chars = ['r', 'w', 'x'];
    for i in (0..3).rev() {
        let shift = i * 3;
        for (j, c) in chars.iter().enumerate() {
            if mode & (1 << (shift + 2 - j)) != 0 {
                s.push(*c);
            } else {
                s.push('-');
            }
        }
    }
    s
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];
    let mut size = bytes as f64;
    for unit in UNITS {
        if size < 1024.0 {
            return if size == size.floor() {
                format!("{:>4}{}", size as u64, unit)
            } else {
                format!("{:>4.1}{}", size, unit)
            };
        }
        size /= 1024.0;
    }
    format!("{:.1}E", size)
}

fn print_columns(names: &[String]) {
    if names.is_empty() {
        return;
    }

    // Simple implementation: try to fit into 80 columns
    let term_width = 80;
    let max_len = names.iter().map(|n| n.len()).max().unwrap_or(0) + 2;
    let cols = std::cmp::max(1, term_width / max_len);

    for (i, name) in names.iter().enumerate() {
        if cols == 1 || (i + 1) % cols == 0 || i == names.len() - 1 {
            println!("{name}");
        } else {
            print!("{name:<width$}", width = max_len);
        }
    }
}
