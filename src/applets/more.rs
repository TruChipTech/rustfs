/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! more — a simple terminal pager.
//!
//! When stdout is not a terminal it behaves like `cat`. On a terminal it shows
//! one screenful at a time, advancing a page on Space and a line on Enter, and
//! quitting on 'q'.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let files: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();

    let is_tty = unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 };
    let rows = terminal_rows().saturating_sub(1).max(1);

    let mut rc = 0;
    if files.is_empty() {
        let stdin = io::stdin();
        page(&mut stdin.lock(), is_tty, rows, None);
    } else {
        let many = files.len() > 1;
        for f in files {
            match File::open(f) {
                Ok(fh) => {
                    let mut r = BufReader::new(fh);
                    let header = if many { Some(f.as_str()) } else { None };
                    if !page(&mut r, is_tty, rows, header) {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("more: {f}: {e}");
                    rc = 1;
                }
            }
        }
    }
    rc
}

/// Returns false if the user asked to quit.
fn page<R: BufRead>(r: &mut R, is_tty: bool, rows: usize, header: Option<&str>) -> bool {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if let Some(name) = header {
        let _ = writeln!(out, "::::::::::::::\n{name}\n::::::::::::::");
    }

    if !is_tty {
        let mut buf = Vec::new();
        let _ = r.read_to_end(&mut buf);
        let _ = out.write_all(&buf);
        return true;
    }

    let mut shown = 0usize;
    let mut line = String::new();
    loop {
        line.clear();
        match r.read_line(&mut line) {
            Ok(0) => return true,
            Ok(_) => {
                let _ = out.write_all(line.as_bytes());
                shown += 1;
                if shown >= rows {
                    let _ = out.flush();
                    match prompt() {
                        Prompt::Quit => return false,
                        Prompt::Page => shown = 0,
                        Prompt::Line => shown = rows - 1,
                    }
                }
            }
            Err(_) => return true,
        }
    }
}

enum Prompt {
    Page,
    Line,
    Quit,
}

fn prompt() -> Prompt {
    // Read a single keypress from the controlling terminal.
    let mut tty = match File::open("/dev/tty") {
        Ok(f) => f,
        Err(_) => return Prompt::Page,
    };
    eprint!("--More--");
    let _ = io::stderr().flush();
    let mut b = [0u8; 1];
    let r = tty.read(&mut b);
    eprint!("\r        \r");
    let _ = io::stderr().flush();
    match r {
        Ok(1) => match b[0] {
            b'q' | b'Q' => Prompt::Quit,
            b'\n' | b'\r' => Prompt::Line,
            _ => Prompt::Page,
        },
        _ => Prompt::Quit,
    }
}

fn terminal_rows() -> usize {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_row > 0 {
            return ws.ws_row as usize;
        }
    }
    24
}
