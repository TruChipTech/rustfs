/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};

pub fn run(args: &[String]) -> i32 {
    let mut num_lines: usize = 10;
    let mut num_bytes: Option<usize> = None;
    let mut follow = false;
    let mut follow_name = false; // -F: reopen on rotation
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                i += 1;
                if i < args.len() {
                    num_lines = args[i].parse().unwrap_or(10);
                }
            }
            "-c" => {
                i += 1;
                if i < args.len() {
                    num_bytes = Some(args[i].parse().unwrap_or(0));
                }
            }
            "-f" | "--follow" | "--follow=descriptor" => follow = true,
            "-F" | "--follow=name" => { follow = true; follow_name = true; }
            arg if arg.starts_with("-n") && arg.len() > 2 => {
                num_lines = arg[2..].parse().unwrap_or(10);
            }
            arg if arg.starts_with("-c") && arg.len() > 2 => {
                num_bytes = Some(arg[2..].parse().unwrap_or(0));
            }
            arg if arg.starts_with('-') && arg.len() > 1 && arg[1..].parse::<usize>().is_ok() => {
                num_lines = arg[1..].parse().unwrap_or(10);
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if files.is_empty() {
        files.push("-".to_string());
    }

    let show_header = files.len() > 1;
    let mut exit_code = 0;

    for (idx, file) in files.iter().enumerate() {
        if show_header {
            if idx > 0 {
                println!();
            }
            println!("==> {file} <==");
        }

        if file == "-" {
            // Read stdin into a buffer and print last N lines
            let stdin = io::stdin();
            let reader = BufReader::new(stdin.lock());
            if let Some(bytes) = num_bytes {
                let mut all = Vec::new();
                let _ = reader.take(1024 * 1024 * 100).read_to_end(&mut all); // 100MB limit
                let start = all.len().saturating_sub(bytes);
                use std::io::Write;
                let _ = io::stdout().write_all(&all[start..]);
            } else {
                let mut ring: VecDeque<String> = VecDeque::with_capacity(num_lines + 1);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            ring.push_back(l);
                            if ring.len() > num_lines {
                                ring.pop_front();
                            }
                        }
                        Err(_) => break,
                    }
                }
                for l in &ring {
                    println!("{l}");
                }
            }
        } else {
            match File::open(file) {
                Ok(f) => {
                    if let Some(bytes) = num_bytes {
                        tail_bytes(&f, bytes);
                    } else {
                        tail_lines(&f, num_lines);
                    }

                    // Follow mode
                    if follow {
                        if follow_name {
                            follow_file_name(file);
                        } else {
                            follow_file(file);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("tail: cannot open '{file}': {e}");
                    exit_code = 1;
                }
            }
        }
    }

    exit_code
}

fn tail_lines(file: &File, n: usize) {
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
    let start = all_lines.len().saturating_sub(n);
    for line in &all_lines[start..] {
        println!("{line}");
    }
}

fn tail_bytes(file: &File, n: usize) {
    use std::io::Write;
    let meta = file.metadata().ok();
    let file_size = meta.map(|m| m.len()).unwrap_or(0);
    let start = (file_size as usize).saturating_sub(n);
    let mut f = BufReader::new(file);
    let _ = f.seek(SeekFrom::Start(start as u64));
    let mut buf = vec![0u8; 8192];
    loop {
        match f.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let _ = io::stdout().write_all(&buf[..n]);
            }
            Err(_) => break,
        }
    }
}

fn follow_file(path: &str) {
    use std::thread;
    use std::time::Duration;

    let mut last_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    loop {
        thread::sleep(Duration::from_secs(1));

        let current_size = match std::fs::metadata(path) {
            Ok(m) => m.len(),
            Err(_) => continue,
        };

        if current_size < last_size { last_size = 0; }

        if current_size > last_size {
            if let Ok(mut f) = File::open(path) {
                let _ = f.seek(SeekFrom::Start(last_size));
                let mut buf = vec![0u8; 8192];
                loop {
                    match f.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            use std::io::Write;
                            let _ = io::stdout().write_all(&buf[..n]);
                        }
                        Err(_) => break,
                    }
                }
                last_size = current_size;
            }
        }
    }
}

fn follow_file_name(path: &str) {
    // Like follow_file but reopens the file when it's replaced (log rotation).
    use std::os::unix::fs::MetadataExt;
    use std::thread;
    use std::time::Duration;

    let get_inode = |p: &str| std::fs::metadata(p).ok().map(|m| m.ino());

    let mut last_inode = get_inode(path);
    let mut last_size: u64 = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    loop {
        thread::sleep(Duration::from_millis(500));

        let cur_inode = get_inode(path);
        let cur_size  = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        // File replaced (different inode) or truncated
        if cur_inode != last_inode || cur_size < last_size {
            eprintln!("tail: '{path}': file truncated or replaced; following new file");
            last_size  = 0;
            last_inode = cur_inode;
        }

        if cur_size > last_size {
            if let Ok(mut f) = File::open(path) {
                let _ = f.seek(SeekFrom::Start(last_size));
                let mut buf = vec![0u8; 8192];
                loop {
                    match f.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            use std::io::Write;
                            let _ = io::stdout().write_all(&buf[..n]);
                        }
                        Err(_) => break,
                    }
                }
                last_size = cur_size;
            }
        }
    }
}
