/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut no_newline = false;
    let mut enable_escapes = false;
    let mut start = 0;

    // Parse leading flags
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "-n" => {
                no_newline = true;
                start = i + 1;
            }
            "-e" => {
                enable_escapes = true;
                start = i + 1;
            }
            "-E" => {
                enable_escapes = false;
                start = i + 1;
            }
            "-ne" | "-en" => {
                no_newline = true;
                enable_escapes = true;
                start = i + 1;
            }
            _ => break,
        }
    }

    let output = args[start..].join(" ");
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if enable_escapes {
        let processed = process_escapes(&output);
        let _ = out.write_all(processed.as_bytes());
    } else {
        let _ = out.write_all(output.as_bytes());
    }

    if !no_newline {
        let _ = out.write_all(b"\n");
    }

    let _ = out.flush();
    0
}

fn process_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('a') => result.push('\x07'),
                Some('b') => result.push('\x08'),
                Some('f') => result.push('\x0C'),
                Some('v') => result.push('\x0B'),
                Some('\\') => result.push('\\'),
                Some('0') => {
                    // Octal escape \0NNN — up to 3 octal digits.
                    let mut val: u8 = 0;
                    for _ in 0..3 {
                        match chars.peek().and_then(|c| c.to_digit(8)) {
                            Some(d) => { val = val.wrapping_mul(8).wrapping_add(d as u8); chars.next(); }
                            None => break,
                        }
                    }
                    result.push(val as char);
                }
                Some('x') => {
                    // Hex escape \xHH — up to 2 hex digits.
                    let mut val: u8 = 0;
                    let mut count = 0;
                    while count < 2 {
                        match chars.peek().and_then(|c| c.to_digit(16)) {
                            Some(d) => { val = val.wrapping_mul(16).wrapping_add(d as u8); chars.next(); count += 1; }
                            None => break,
                        }
                    }
                    if count > 0 {
                        result.push(val as char);
                    } else {
                        result.push_str("\\x");
                    }
                }
                Some('c') => break, // \c stops output
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}
