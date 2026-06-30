/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> i32 {
    let mut reverse = false;
    let mut cols: usize = 16;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-r" | "--reverse" => reverse = true,
            "-c" | "--cols" => {
                i += 1;
                if i < args.len() {
                    cols = args[i].parse().unwrap_or(16);
                }
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if reverse {
        return reverse_xxd(&files);
    }

    let mut input = Vec::new();
    if files.is_empty() || (files.len() == 1 && files[0] == "-") {
        if let Err(e) = io::stdin().read_to_end(&mut input) {
            eprintln!("xxd: read error: {e}");
            return 1;
        }
    } else {
        for f in &files {
            match std::fs::read(f) {
                Ok(data) => input.extend(data),
                Err(e) => {
                    eprintln!("xxd: {f}: {e}");
                    return 1;
                }
            }
        }
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for (i, chunk) in input.chunks(cols).enumerate() {
        let offset = i * cols;
        let _ = write!(out, "{:08x}: ", offset);

        // Hex part
        for (j, &byte) in chunk.iter().enumerate() {
            let _ = write!(out, "{:02x}", byte);
            if j % 2 == 1 {
                let _ = write!(out, " ");
            }
        }

        // Padding
        for j in chunk.len()..cols {
            let _ = write!(out, "  ");
            if j % 2 == 1 {
                let _ = write!(out, " ");
            }
        }

        let _ = write!(out, " ");

        // ASCII part
        for &byte in chunk {
            let c = if (0x20..=0x7e).contains(&byte) {
                byte as char
            } else {
                '.'
            };
            let _ = write!(out, "{c}");
        }

        let _ = writeln!(out);
    }

    let _ = out.flush();
    0
}

fn reverse_xxd(files: &[String]) -> i32 {
    let lines = super::input_lines(files);
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in &lines {
        // Parse xxd output: skip offset, read hex bytes
        if let Some(colon_pos) = line.find(':') {
            let hex_part = &line[colon_pos + 1..];
            // Find the ASCII part separator (double space)
            let hex_end = hex_part.find("  ").unwrap_or(hex_part.len());
            let hex_str: String = hex_part[..hex_end]
                .chars()
                .filter(|c| c.is_ascii_hexdigit())
                .collect();

            for i in (0..hex_str.len()).step_by(2) {
                if i + 1 < hex_str.len() {
                    if let Ok(byte) = u8::from_str_radix(&hex_str[i..i + 2], 16) {
                        let _ = out.write_all(&[byte]);
                    }
                }
            }
        }
    }

    let _ = out.flush();
    0
}
