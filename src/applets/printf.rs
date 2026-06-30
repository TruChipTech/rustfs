/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        return 0;
    }

    let format = &args[0];
    let format_args = &args[1..];
    let mut arg_idx = 0;

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => { let _ = out.write_all(b"\n"); }
                Some('t') => { let _ = out.write_all(b"\t"); }
                Some('r') => { let _ = out.write_all(b"\r"); }
                Some('\\') => { let _ = out.write_all(b"\\"); }
                Some('a') => { let _ = out.write_all(b"\x07"); }
                Some('b') => { let _ = out.write_all(b"\x08"); }
                Some('f') => { let _ = out.write_all(b"\x0c"); }
                Some('v') => { let _ = out.write_all(b"\x0b"); }
                Some('0') => {
                    let mut oct = String::new();
                    for _ in 0..3 {
                        if let Some(&d) = chars.peek() {
                            if ('0'..='7').contains(&d) {
                                oct.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    let val = u8::from_str_radix(&oct, 8).unwrap_or(0);
                    let _ = out.write_all(&[val]);
                }
                Some(other) => {
                    let _ = write!(out, "\\{other}");
                }
                None => { let _ = out.write_all(b"\\"); }
            }
        } else if c == '%' {
            // Format specifier
            let mut spec = String::from('%');
            // Read flags, width, precision
            while let Some(&ch) = chars.peek() {
                if ch == '-' || ch == '+' || ch == ' ' || ch == '0' || ch == '#'
                    || ch.is_ascii_digit() || ch == '.'
                {
                    spec.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }

            if let Some(conv) = chars.next() {
                let arg_val = if arg_idx < format_args.len() {
                    &format_args[arg_idx]
                } else {
                    ""
                };
                arg_idx += 1;

                match conv {
                    's' => { let _ = write!(out, "{arg_val}"); }
                    'd' | 'i' => {
                        let n: i64 = arg_val.parse().unwrap_or(0);
                        let _ = write!(out, "{n}");
                    }
                    'u' => {
                        let n: u64 = arg_val.parse().unwrap_or(0);
                        let _ = write!(out, "{n}");
                    }
                    'o' => {
                        let n: u64 = arg_val.parse().unwrap_or(0);
                        let _ = write!(out, "{n:o}");
                    }
                    'x' => {
                        let n: u64 = arg_val.parse().unwrap_or(0);
                        let _ = write!(out, "{n:x}");
                    }
                    'X' => {
                        let n: u64 = arg_val.parse().unwrap_or(0);
                        let _ = write!(out, "{n:X}");
                    }
                    'f' => {
                        let n: f64 = arg_val.parse().unwrap_or(0.0);
                        let _ = write!(out, "{n:.6}");
                    }
                    'c' => {
                        let ch = arg_val.chars().next().unwrap_or('\0');
                        let _ = write!(out, "{ch}");
                    }
                    '%' => {
                        let _ = out.write_all(b"%");
                        arg_idx -= 1; // %% doesn't consume an argument
                    }
                    _ => {
                        let _ = write!(out, "%{conv}");
                        arg_idx -= 1;
                    }
                }
            }
        } else {
            let _ = write!(out, "{c}");
        }
    }

    let _ = out.flush();
    0
}
