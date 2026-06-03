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
    if args.is_empty() {
        eprintln!("stat: missing operand");
        return 1;
    }

    let mut exit_code = 0;

    for file in args {
        let path = Path::new(file);
        match fs::symlink_metadata(path) {
            Ok(meta) => {
                println!("  File: {file}");

                let file_type = if meta.is_dir() {
                    "directory"
                } else if meta.file_type().is_symlink() {
                    "symbolic link"
                } else {
                    "regular file"
                };
                println!("  Size: {:<15} {file_type}", meta.len());

                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    println!(
                        "Device: {:x}h/{:}d\tInode: {:<10} Links: {}",
                        meta.dev(),
                        meta.dev(),
                        meta.ino(),
                        meta.nlink()
                    );
                    println!(
                        "Access: ({:04o}/{})\tUid: {:5}\tGid: {:5}",
                        meta.mode() & 0o7777,
                        format_mode(meta.mode()),
                        meta.uid(),
                        meta.gid()
                    );
                }

                if let Ok(accessed) = meta.accessed() {
                    let dt: chrono::DateTime<chrono::Local> = accessed.into();
                    println!("Access: {}", dt.format("%Y-%m-%d %H:%M:%S%.9f %z"));
                }
                if let Ok(modified) = meta.modified() {
                    let dt: chrono::DateTime<chrono::Local> = modified.into();
                    println!("Modify: {}", dt.format("%Y-%m-%d %H:%M:%S%.9f %z"));
                }
                if let Ok(created) = meta.created() {
                    let dt: chrono::DateTime<chrono::Local> = created.into();
                    println!(" Birth: {}", dt.format("%Y-%m-%d %H:%M:%S%.9f %z"));
                }
            }
            Err(e) => {
                eprintln!("stat: cannot stat '{file}': {e}");
                exit_code = 1;
            }
        }
    }

    exit_code
}

#[cfg(unix)]
fn format_mode(mode: u32) -> String {
    let file_type = match mode & 0o170000 {
        0o140000 => 's',
        0o120000 => 'l',
        0o100000 => '-',
        0o060000 => 'b',
        0o040000 => 'd',
        0o020000 => 'c',
        0o010000 => 'p',
        _ => '?',
    };

    let mut s = String::with_capacity(10);
    s.push(file_type);

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
