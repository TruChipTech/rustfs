/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! less — file pager (view file contents one screen at a time)

use std::fs;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut files: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                eprintln!("Usage: less [file...]");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
    }

    let content = if files.is_empty() {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            eprintln!("less: {e}");
            return 1;
        }
        buf
    } else {
        let mut combined = String::new();
        for file in &files {
            match fs::read_to_string(file) {
                Ok(c) => {
                    if files.len() > 1 {
                        combined.push_str(&format!(":::::::::::\n{file}\n:::::::::::\n"));
                    }
                    combined.push_str(&c);
                }
                Err(e) => {
                    eprintln!("less: {file}: {e}");
                    return 1;
                }
            }
        }
        combined
    };

    let lines: Vec<&str> = content.lines().collect();
    let term_rows = get_terminal_rows();
    let page_size = if term_rows > 1 { term_rows - 1 } else { 24 };
    let mut offset = 0;

    // Open /dev/tty for input if stdin is a pipe
    let tty = std::fs::File::open("/dev/tty").ok();

    loop {
        // Display a page
        let end = (offset + page_size).min(lines.len());
        for line in &lines[offset..end] {
            println!("{line}");
        }

        if end >= lines.len() {
            print!("(END)");
            let _ = io::stdout().flush();
            // Wait for 'q'
            let mut buf = [0u8; 1];
            if let Some(ref tty_file) = tty {
                let mut t = tty_file.try_clone().unwrap();
                let _ = t.read(&mut buf);
            } else {
                let _ = io::stdin().read(&mut buf);
            }
            println!();
            break;
        }

        print!(":");
        let _ = io::stdout().flush();

        let mut buf = [0u8; 1];
        let read_result = if let Some(ref tty_file) = tty {
            let mut t = tty_file.try_clone().unwrap();
            t.read(&mut buf)
        } else {
            io::stdin().read(&mut buf)
        };

        match read_result {
            Ok(0) => break,
            Err(_) => break,
            _ => {}
        }

        match buf[0] {
            b'q' | b'Q' => break,
            b' ' | b'f' => offset = end,
            b'b' => offset = offset.saturating_sub(page_size),
            b'\n' | b'j' => offset += 1,
            b'k' => offset = offset.saturating_sub(1),
            b'g' => offset = 0,
            b'G' => offset = lines.len().saturating_sub(page_size),
            _ => offset = end,
        }
    }

    0
}

fn get_terminal_rows() -> usize {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    if unsafe { libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) } == 0 && ws.ws_row > 0 {
        ws.ws_row as usize
    } else {
        24
    }
}
