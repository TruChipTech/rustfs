/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::path::Path;
use walkdir::WalkDir;

pub fn run(args: &[String]) -> i32 {
    let mut summarize = false;
    let mut human_readable = false;
    let mut total = false;
    let mut paths = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--summarize" => summarize = true,
            "-h" | "--human-readable" => human_readable = true,
            "-c" | "--total" => total = true,
            _ => paths.push(args[i].clone()),
        }
        i += 1;
    }

    if paths.is_empty() {
        paths.push(".".to_string());
    }

    let mut grand_total: u64 = 0;
    let mut exit_code = 0;

    for p in &paths {
        let path = Path::new(p);
        if !path.exists() {
            eprintln!("du: cannot access '{p}': No such file or directory");
            exit_code = 1;
            continue;
        }

        if path.is_file() {
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            let blocks = (size + 1023) / 1024;
            grand_total += blocks;
            print_size(blocks, p, human_readable);
            continue;
        }

        let mut dir_total: u64 = 0;

        for entry in WalkDir::new(path).follow_links(false) {
            match entry {
                Ok(e) => {
                    if let Ok(meta) = e.metadata() {
                        let size = meta.len();
                        let blocks = (size + 1023) / 1024;

                        if !summarize && e.path().is_dir() {
                            // Calculate directory subtotal
                            let sub: u64 = WalkDir::new(e.path())
                                .follow_links(false)
                                .into_iter()
                                .filter_map(|ee| ee.ok())
                                .filter_map(|ee| ee.metadata().ok())
                                .map(|m| (m.len() + 1023) / 1024)
                                .sum();
                            print_size(sub, &e.path().display().to_string(), human_readable);
                        }

                        dir_total += blocks;
                    }
                }
                Err(e) => {
                    eprintln!("du: {e}");
                    exit_code = 1;
                }
            }
        }

        if summarize {
            print_size(dir_total, p, human_readable);
        }

        grand_total += dir_total;
    }

    if total {
        print_size(grand_total, "total", human_readable);
    }

    exit_code
}

fn print_size(blocks: u64, name: &str, human_readable: bool) {
    if human_readable {
        println!("{}\t{name}", human_size(blocks * 1024));
    } else {
        println!("{blocks}\t{name}");
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut size = bytes as f64;
    for unit in UNITS {
        if size < 1024.0 {
            return format!("{:.1}{unit}", size);
        }
        size /= 1024.0;
    }
    format!("{:.1}P", size)
}
