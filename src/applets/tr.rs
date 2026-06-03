/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut delete = false;
    let mut squeeze = false;
    let mut complement = false;
    let mut operands = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--delete" => delete = true,
            "-s" | "--squeeze-repeats" => squeeze = true,
            "-c" | "--complement" => complement = true,
            arg if arg.starts_with('-') && arg.len() > 1 => {
                for c in arg[1..].chars() {
                    match c {
                        'd' => delete = true,
                        's' => squeeze = true,
                        'c' | 'C' => complement = true,
                        _ => {
                            eprintln!("tr: unknown option '-{c}'");
                            return 1;
                        }
                    }
                }
            }
            _ => operands.push(args[i].clone()),
        }
        i += 1;
    }

    if operands.is_empty() {
        eprintln!("tr: missing operand");
        return 1;
    }

    let set1 = expand_set(&operands[0]);
    let set2 = if operands.len() > 1 {
        expand_set(&operands[1])
    } else {
        Vec::new()
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut input = Vec::new();
    let _ = stdin.lock().read_to_end(&mut input);

    let mut last_char: Option<u8> = None;

    for &byte in &input {
        let in_set1 = set1.contains(&byte);
        let in_set = if complement { !in_set1 } else { in_set1 };

        if delete {
            if !in_set {
                if squeeze {
                    if last_char == Some(byte) {
                        continue;
                    }
                    last_char = Some(byte);
                }
                let _ = out.write_all(&[byte]);
            }
        } else if !set2.is_empty() {
            if in_set {
                // Find the index in set1 and map to set2
                let idx = if complement {
                    0 // for complement, map all non-set1 chars to set2[0]
                } else {
                    set1.iter().position(|&b| b == byte).unwrap_or(0)
                };
                let replacement = set2[std::cmp::min(idx, set2.len() - 1)];

                if squeeze {
                    if last_char == Some(replacement) {
                        continue;
                    }
                    last_char = Some(replacement);
                }
                let _ = out.write_all(&[replacement]);
            } else {
                if squeeze {
                    if last_char == Some(byte) && set2.contains(&byte) {
                        continue;
                    }
                    last_char = Some(byte);
                }
                let _ = out.write_all(&[byte]);
            }
        } else if squeeze && in_set {
            if last_char != Some(byte) {
                let _ = out.write_all(&[byte]);
                last_char = Some(byte);
            }
        } else {
            let _ = out.write_all(&[byte]);
            last_char = Some(byte);
        }
    }

    let _ = out.flush();
    0
}

fn expand_set(spec: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = spec.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if i + 2 < bytes.len() && bytes[i + 1] == b'-' {
            // Range: a-z
            let start = bytes[i];
            let end = bytes[i + 2];
            if start <= end {
                for b in start..=end {
                    result.push(b);
                }
            }
            i += 3;
        } else if bytes[i] == b'\\' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'n' => result.push(b'\n'),
                b't' => result.push(b'\t'),
                b'r' => result.push(b'\r'),
                b'\\' => result.push(b'\\'),
                b'a' => result.push(0x07),
                b'b' => result.push(0x08),
                b'f' => result.push(0x0C),
                b'v' => result.push(0x0B),
                _ => result.push(bytes[i + 1]),
            }
            i += 2;
        } else if bytes[i] == b'[' && i + 1 < bytes.len() && bytes[i + 1] == b':' {
            // POSIX class [:alpha:] etc
            if let Some(end) = spec[i..].find(":]") {
                let class = &spec[i + 2..i + end];
                match class {
                    "alpha" => result.extend(b'a'..=b'z'),
                    "upper" => result.extend(b'A'..=b'Z'),
                    "lower" => result.extend(b'a'..=b'z'),
                    "digit" => result.extend(b'0'..=b'9'),
                    "alnum" => {
                        result.extend(b'a'..=b'z');
                        result.extend(b'A'..=b'Z');
                        result.extend(b'0'..=b'9');
                    }
                    "space" => result.extend_from_slice(b" \t\n\r\x0b\x0c"),
                    "blank" => result.extend_from_slice(b" \t"),
                    _ => {}
                }
                i += end + 2;
            } else {
                result.push(bytes[i]);
                i += 1;
            }
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }

    result
}
