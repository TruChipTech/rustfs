/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! catv — display nonprinting characters visibly (cat -v with -e/-t).

use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut show_ends = false; // -e: mark line ends with '$'
    let mut show_tabs = false; // -t: show tabs as ^I
    let mut files: Vec<String> = Vec::new();

    for a in args {
        match a.as_str() {
            "-e" => show_ends = true,
            "-t" => show_tabs = true,
            "-v" => {} // nonprinting display is always on for catv
            "--help" => {
                eprintln!("Usage: catv [-etv] [FILE]...");
                return 0;
            }
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        'e' => show_ends = true,
                        't' => show_tabs = true,
                        'v' => {}
                        _ => {
                            eprintln!("catv: invalid option -- '{c}'");
                            return 1;
                        }
                    }
                }
            }
            _ => files.push(a.clone()),
        }
    }

    let inputs: Vec<String> = if files.is_empty() {
        vec!["-".to_string()]
    } else {
        files
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut rc = 0;
    for f in &inputs {
        let mut reader: Box<dyn Read> = if f == "-" {
            Box::new(io::stdin())
        } else {
            match std::fs::File::open(f) {
                Ok(fh) => Box::new(fh),
                Err(e) => {
                    eprintln!("catv: {f}: {e}");
                    rc = 1;
                    continue;
                }
            }
        };
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    for &b in &buf[..n] {
                        if write_byte(&mut out, b, show_ends, show_tabs).is_err() {
                            return 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("catv: {f}: {e}");
                    rc = 1;
                    break;
                }
            }
        }
    }
    0.max(rc)
}

fn write_byte<W: Write>(out: &mut W, b: u8, show_ends: bool, show_tabs: bool) -> io::Result<()> {
    match b {
        b'\n' => {
            if show_ends {
                out.write_all(b"$")?;
            }
            out.write_all(b"\n")
        }
        b'\t' => {
            if show_tabs {
                out.write_all(b"^I")
            } else {
                out.write_all(b"\t")
            }
        }
        0..=31 => {
            out.write_all(&[b'^', b + 64])
        }
        127 => out.write_all(b"^?"),
        128..=255 => {
            out.write_all(b"M-")?;
            let c = b - 128;
            match c {
                b'\t' => out.write_all(b"^I"),
                b'\n' => out.write_all(b"^J"),
                0..=31 => out.write_all(&[b'^', c + 64]),
                127 => out.write_all(b"^?"),
                _ => out.write_all(&[c]),
            }
        }
        _ => out.write_all(&[b]),
    }
}
