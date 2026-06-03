/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ftpput — upload a file via FTP

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut user = "anonymous".to_string();
    let mut pass = "ftp@".to_string();
    let mut port = 21u16;
    let mut positional: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-u" => { i += 1; if i < args.len() { user = args[i].clone(); } }
            "-p" => { i += 1; if i < args.len() { pass = args[i].clone(); } }
            "-P" => { i += 1; if i < args.len() { port = args[i].parse().unwrap_or(21); } }
            "-h" | "--help" => {
                eprintln!("Usage: ftpput [-u USER] [-p PASS] [-P PORT] HOST REMOTE LOCAL");
                return 0;
            }
            s => positional.push(s.to_string()),
        }
        i += 1;
    }

    if positional.len() < 3 {
        eprintln!("Usage: ftpput [-u USER] [-p PASS] [-P PORT] HOST REMOTE LOCAL");
        return 1;
    }

    let host = &positional[0];
    let remote = &positional[1];
    let local = &positional[2];

    let data = match fs::read(local) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("ftpput: cannot read {local}: {e}");
            return 1;
        }
    };

    let mut stream = match TcpStream::connect(format!("{host}:{port}")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ftpput: cannot connect to {host}:{port}: {e}");
            return 1;
        }
    };

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut response = String::new();

    // Read greeting
    let _ = reader.read_line(&mut response);

    // Login
    let _ = stream.write_all(format!("USER {user}\r\n").as_bytes());
    response.clear();
    let _ = reader.read_line(&mut response);

    let _ = stream.write_all(format!("PASS {pass}\r\n").as_bytes());
    response.clear();
    let _ = reader.read_line(&mut response);

    // Binary mode
    let _ = stream.write_all(b"TYPE I\r\n");
    response.clear();
    let _ = reader.read_line(&mut response);

    // Passive mode
    let _ = stream.write_all(b"PASV\r\n");
    response.clear();
    let _ = reader.read_line(&mut response);

    let data_addr = match parse_pasv(&response) {
        Some(a) => a,
        None => {
            eprintln!("ftpput: failed to parse PASV response");
            return 1;
        }
    };

    // Connect data channel
    let mut data_stream = match TcpStream::connect(&data_addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ftpput: data connection failed: {e}");
            return 1;
        }
    };

    // Store file
    let _ = stream.write_all(format!("STOR {remote}\r\n").as_bytes());
    response.clear();
    let _ = reader.read_line(&mut response);

    if !response.starts_with("150") && !response.starts_with("125") {
        eprintln!("ftpput: STOR failed: {response}");
        return 1;
    }

    if let Err(e) = data_stream.write_all(&data) {
        eprintln!("ftpput: upload failed: {e}");
        return 1;
    }
    drop(data_stream);

    response.clear();
    let _ = reader.read_line(&mut response);
    let _ = stream.write_all(b"QUIT\r\n");
    0
}

fn parse_pasv(response: &str) -> Option<String> {
    let start = response.find('(')?;
    let end = response.find(')')?;
    let nums: Vec<u16> = response[start + 1..end]
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if nums.len() == 6 {
        let ip = format!("{}.{}.{}.{}", nums[0], nums[1], nums[2], nums[3]);
        let port = nums[4] * 256 + nums[5];
        Some(format!("{ip}:{port}"))
    } else {
        None
    }
}
