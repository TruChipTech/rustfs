/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! httpd — minimal HTTP server

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;

pub fn run(args: &[String]) -> i32 {
    let mut port = 80u16;
    let mut root = "/var/www".to_string();
    let mut foreground = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => { i += 1; if i < args.len() { port = args[i].parse().unwrap_or(80); } }
            "-h" => { i += 1; if i < args.len() { root = args[i].clone(); } }
            "-f" => foreground = true,
            "--help" => {
                eprintln!("Usage: httpd [-f] [-p PORT] [-h HOME]");
                return 0;
            }
            s if !s.starts_with('-') => root = s.to_string(),
            _ => {}
        }
        i += 1;
    }

    if !foreground {
        // Daemonize
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            eprintln!("httpd: fork failed");
            return 1;
        }
        if pid > 0 {
            return 0; // Parent exits
        }
        unsafe { libc::setsid(); }
    }

    let listener = match TcpListener::bind(format!("0.0.0.0:{port}")) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("httpd: cannot bind to port {port}: {e}");
            return 1;
        }
    };

    eprintln!("httpd: serving {root} on port {port}");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };

        let reader = BufReader::new(match stream.try_clone() {
            Ok(s) => s,
            Err(_) => continue,
        });

        // Read request line
        let mut lines = reader.lines();
        let request_line = match lines.next() {
            Some(Ok(l)) => l,
            _ => continue,
        };

        // Skip headers
        for line in &mut lines {
            match line {
                Ok(l) if l.is_empty() || l == "\r" => break,
                Err(_) => break,
                _ => {}
            }
        }

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            let _ = stream.write_all(b"HTTP/1.0 400 Bad Request\r\n\r\n");
            continue;
        }

        let method = parts[0];
        let path = parts[1];

        if method != "GET" && method != "HEAD" {
            let _ = stream.write_all(b"HTTP/1.0 405 Method Not Allowed\r\n\r\n");
            continue;
        }

        // Sanitize path to prevent directory traversal
        let decoded_path = path.replace("%20", " ");
        let clean_path = decoded_path.trim_start_matches('/');
        let file_path = PathBuf::from(&root).join(clean_path);

        // Ensure path doesn't escape root
        let canonical_root = fs::canonicalize(&root).unwrap_or_else(|_| PathBuf::from(&root));
        let canonical_file = match fs::canonicalize(&file_path) {
            Ok(p) => p,
            Err(_) => {
                let _ = stream.write_all(b"HTTP/1.0 404 Not Found\r\nContent-Type: text/plain\r\n\r\n404 Not Found\n");
                continue;
            }
        };

        if !canonical_file.starts_with(&canonical_root) {
            let _ = stream.write_all(b"HTTP/1.0 403 Forbidden\r\n\r\n");
            continue;
        }

        // Serve directory index or file
        let serve_path = if canonical_file.is_dir() {
            let index = canonical_file.join("index.html");
            if index.exists() { index } else {
                // Directory listing
                let mut listing = String::from("<html><body><ul>\n");
                if let Ok(entries) = fs::read_dir(&canonical_file) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        listing.push_str(&format!("<li><a href=\"{name}\">{name}</a></li>\n"));
                    }
                }
                listing.push_str("</ul></body></html>\n");
                let header = format!(
                    "HTTP/1.0 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                    listing.len()
                );
                let _ = stream.write_all(header.as_bytes());
                if method == "GET" {
                    let _ = stream.write_all(listing.as_bytes());
                }
                continue;
            }
        } else {
            canonical_file
        };

        match fs::read(&serve_path) {
            Ok(content) => {
                let mime = guess_mime(serve_path.to_str().unwrap_or(""));
                let header = format!(
                    "HTTP/1.0 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\n\r\n",
                    content.len()
                );
                let _ = stream.write_all(header.as_bytes());
                if method == "GET" {
                    let _ = stream.write_all(&content);
                }
            }
            Err(_) => {
                let _ = stream.write_all(b"HTTP/1.0 404 Not Found\r\n\r\n404 Not Found\n");
            }
        }
    }
    0
}

fn guess_mime(path: &str) -> &'static str {
    if path.ends_with(".html") || path.ends_with(".htm") { "text/html" }
    else if path.ends_with(".css") { "text/css" }
    else if path.ends_with(".js") { "application/javascript" }
    else if path.ends_with(".json") { "application/json" }
    else if path.ends_with(".png") { "image/png" }
    else if path.ends_with(".jpg") || path.ends_with(".jpeg") { "image/jpeg" }
    else if path.ends_with(".gif") { "image/gif" }
    else if path.ends_with(".svg") { "image/svg+xml" }
    else if path.ends_with(".txt") { "text/plain" }
    else { "application/octet-stream" }
}
