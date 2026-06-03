/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! logread — read and display syslog ring buffer messages

use std::fs;
use std::io::{self, BufRead};

pub fn run(args: &[String]) -> i32 {
    let mut follow = false;
    let mut log_file = "/var/log/messages".to_string();

    for arg in args {
        match arg.as_str() {
            "-f" => follow = true,
            "-h" | "--help" => {
                eprintln!("Usage: logread [-f]");
                return 0;
            }
            s if !s.starts_with('-') => log_file = s.to_string(),
            _ => {}
        }
    }

    if follow {
        // Tail follow mode
        match fs::File::open(&log_file) {
            Ok(file) => {
                let reader = io::BufReader::new(file);
                // Print existing content
                for line in reader.lines() {
                    match line {
                        Ok(l) => println!("{l}"),
                        Err(_) => break,
                    }
                }
                // Keep reading (like tail -f)
                // Re-open and seek to end
                match fs::File::open(&log_file) {
                    Ok(f) => {
                        use std::io::{Read, Seek, SeekFrom};
                        let mut f = f;
                        let _ = f.seek(SeekFrom::End(0));
                        let mut buf = [0u8; 4096];
                        loop {
                            match f.read(&mut buf) {
                                Ok(0) => {
                                    std::thread::sleep(std::time::Duration::from_secs(1));
                                }
                                Ok(n) => {
                                    let text = String::from_utf8_lossy(&buf[..n]);
                                    print!("{text}");
                                    let _ = io::stdout().lock().flush();
                                }
                                Err(_) => break,
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("logread: {e}");
                        return 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("logread: cannot open {log_file}: {e}");
                return 1;
            }
        }
    } else {
        // One-shot read
        match fs::read_to_string(&log_file) {
            Ok(content) => print!("{content}"),
            Err(e) => {
                eprintln!("logread: cannot open {log_file}: {e}");
                return 1;
            }
        }
    }

    0
}

use std::io::Write;
