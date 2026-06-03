/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ftpd — minimal FTP server

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::fs;
use std::path::PathBuf;

pub fn run(args: &[String]) -> i32 {
    let mut root = "/srv/ftp".to_string();
    let mut port = 21u16;
    let mut anonymous_only = true;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => { i += 1; if i < args.len() { port = args[i].parse().unwrap_or(21); } }
            "-w" => anonymous_only = false,
            "-h" | "--help" => {
                eprintln!("Usage: ftpd [-p PORT] [-w] [ROOT_DIR]");
                return 0;
            }
            s if !s.starts_with('-') => root = s.to_string(),
            _ => {}
        }
        i += 1;
    }

    let listener = match TcpListener::bind(format!("0.0.0.0:{port}")) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("ftpd: cannot bind to port {port}: {e}");
            return 1;
        }
    };

    eprintln!("ftpd: listening on port {port}, root={root}, anonymous={anonymous_only}");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };

        let root = root.clone();
        let _ = stream.write_all(b"220 RustFS ftpd ready\r\n");

        let reader = BufReader::new(stream.try_clone().unwrap_or_else(|_| stream.try_clone().unwrap()));
        let mut cwd = PathBuf::from("/");

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let parts: Vec<&str> = line.trim().splitn(2, ' ').collect();
            let cmd = parts[0].to_uppercase();
            let arg = parts.get(1).unwrap_or(&"");

            match cmd.as_str() {
                "USER" => { let _ = stream.write_all(b"230 Login successful\r\n"); }
                "PASS" => { let _ = stream.write_all(b"230 Login successful\r\n"); }
                "SYST" => { let _ = stream.write_all(b"215 UNIX Type: L8\r\n"); }
                "PWD" | "XPWD" => {
                    let _ = stream.write_all(format!("257 \"{}\"\r\n", cwd.display()).as_bytes());
                }
                "CWD" | "XCWD" => {
                    let new_path = cwd.join(arg);
                    let full = PathBuf::from(&root).join(new_path.strip_prefix("/").unwrap_or(&new_path));
                    if full.is_dir() {
                        cwd = new_path;
                        let _ = stream.write_all(b"250 OK\r\n");
                    } else {
                        let _ = stream.write_all(b"550 Failed to change directory\r\n");
                    }
                }
                "LIST" => {
                    let _ = stream.write_all(b"150 Opening data connection\r\n");
                    let full = PathBuf::from(&root).join(cwd.strip_prefix("/").unwrap_or(&cwd));
                    if let Ok(entries) = fs::read_dir(&full) {
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let _ = stream.write_all(format!("{name}\r\n").as_bytes());
                        }
                    }
                    let _ = stream.write_all(b"226 Transfer complete\r\n");
                }
                "TYPE" => { let _ = stream.write_all(b"200 Type set\r\n"); }
                "QUIT" => {
                    let _ = stream.write_all(b"221 Goodbye\r\n");
                    break;
                }
                "NOOP" => { let _ = stream.write_all(b"200 OK\r\n"); }
                _ => { let _ = stream.write_all(b"502 Command not implemented\r\n"); }
            }
        }
    }
    0
}
