/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs::File;
use std::io::{self, BufReader, Read};

pub fn run(args: &[String]) -> i32 {
    let mut fmt = Fmt::OctalWord; // default: -o
    let mut addr_radix = 'o';
    let mut byte_limit: Option<usize> = None;
    let mut skip_bytes: usize = 0;
    let mut bytes_per_row: usize = 16;
    let mut verbose = false;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-b"               => fmt = Fmt::OctalByte,
            "-c"               => fmt = Fmt::Char,
            "-d"               => fmt = Fmt::DecWord,
            "-o"               => fmt = Fmt::OctalWord,
            "-x" | "-h"        => fmt = Fmt::HexWord,
            "-v" | "--output-duplicates" => verbose = true,
            "-A" => {
                i += 1;
                if i < args.len() { addr_radix = args[i].chars().next().unwrap_or('o'); }
            }
            "-N" | "--read-bytes" => {
                i += 1;
                if i < args.len() { byte_limit = parse_count(&args[i]); }
            }
            "-j" | "--skip-bytes" => {
                i += 1;
                if i < args.len() { skip_bytes = parse_count(&args[i]).unwrap_or(0); }
            }
            "-t" | "--format" => {
                i += 1;
                if i < args.len() { fmt = parse_fmt(&args[i]); }
            }
            "-w" | "--width" => {
                i += 1;
                if i < args.len() { bytes_per_row = args[i].parse().unwrap_or(16); }
            }
            s if s.starts_with("-N") && s.len() > 2 => byte_limit = parse_count(&s[2..]),
            s if s.starts_with("-j") && s.len() > 2 => skip_bytes = parse_count(&s[2..]).unwrap_or(0),
            s if !s.starts_with('-') || s == "-"    => files.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    if files.is_empty() { files.push("-".to_string()); }

    let mut data: Vec<u8> = Vec::new();
    for file in &files {
        let mut reader: Box<dyn Read> = if file == "-" {
            Box::new(io::stdin())
        } else {
            match File::open(file) {
                Ok(f) => Box::new(BufReader::new(f)),
                Err(e) => { eprintln!("od: {file}: {e}"); return 1; }
            }
        };
        if reader.read_to_end(&mut data).is_err() { continue; }
    }

    let data = &data[skip_bytes.min(data.len())..];
    let data = if let Some(lim) = byte_limit { &data[..lim.min(data.len())] } else { data };

    let mut offset = skip_bytes;
    let mut prev: Option<Vec<u8>> = None;
    let mut star = false;

    for chunk in data.chunks(bytes_per_row.max(1)) {
        if !verbose
            && prev.as_deref() == Some(chunk) {
                if !star { println!("*"); star = true; }
                offset += chunk.len();
                continue;
            }
        star = false;
        print!("{}", addr_str(offset, addr_radix));
        print_row(chunk, &fmt, bytes_per_row);
        prev = Some(chunk.to_vec());
        offset += chunk.len();
    }

    // Final address line
    if addr_radix != 'n' {
        println!("{}", addr_str(offset, addr_radix));
    }

    0
}

#[derive(Clone)]
enum Fmt {
    OctalByte,
    OctalWord,
    HexByte,
    HexWord,
    DecWord,
    Char,
}

fn parse_fmt(s: &str) -> Fmt {
    match s {
        "o1" | "o"        => Fmt::OctalByte,
        "o2"              => Fmt::OctalWord,
        "x1"              => Fmt::HexByte,
        "x" | "x2"        => Fmt::HexWord,
        "d" | "d2" | "u2" => Fmt::DecWord,
        "c"               => Fmt::Char,
        _                 => Fmt::OctalByte,
    }
}

fn parse_count(s: &str) -> Option<usize> {
    if s.is_empty() { return None; }
    let (num, mult): (&str, usize) = if s.ends_with('K') || s.ends_with('k') {
        (&s[..s.len()-1], 1024)
    } else if s.ends_with('M') || s.ends_with('m') {
        (&s[..s.len()-1], 1024 * 1024)
    } else if s.ends_with('G') || s.ends_with('g') {
        (&s[..s.len()-1], 1024 * 1024 * 1024)
    } else if s.starts_with("0x") || s.starts_with("0X") {
        return usize::from_str_radix(&s[2..], 16).ok();
    } else if s.starts_with('0') && s.len() > 1 {
        return usize::from_str_radix(&s[1..], 8).ok();
    } else {
        (s, 1)
    };
    num.parse::<usize>().ok().map(|n| n * mult)
}

fn addr_str(offset: usize, radix: char) -> String {
    match radix {
        'd' => format!("{:07}", offset),
        'x' => format!("{:07x}", offset),
        'n' => String::new(),
        _   => format!("{:07o}", offset),
    }
}

fn print_row(chunk: &[u8], fmt: &Fmt, row_width: usize) {
    match fmt {
        Fmt::OctalByte => {
            for &b in chunk { print!(" {:03o}", b); }
        }
        Fmt::OctalWord => {
            for pair in chunk.chunks(2) {
                let w = le16(pair);
                print!(" {:06o}", w);
            }
            // Pad for short last row
            let words_printed = chunk.len().div_ceil(2);
            let words_in_row  = row_width.div_ceil(2);
            for _ in words_printed..words_in_row {
                print!("       "); // 7 spaces
            }
        }
        Fmt::HexByte => {
            for &b in chunk { print!(" {:02x}", b); }
        }
        Fmt::HexWord => {
            for pair in chunk.chunks(2) {
                let w = le16(pair);
                print!(" {:04x}", w);
            }
        }
        Fmt::DecWord => {
            for pair in chunk.chunks(2) {
                let w = le16(pair);
                print!(" {:05}", w);
            }
        }
        Fmt::Char => {
            for &b in chunk {
                let s = match b {
                    0 => "  \\0", 7 => "  \\a", 8 => "  \\b",
                    9 => "  \\t", 10 => "  \\n", 11 => "  \\v",
                    12 => "  \\f", 13 => "  \\r", b'\\' => "  \\\\",
                    0x20..=0x7e => { print!("   {}", b as char); continue; }
                    _ => { print!(" {:03o}", b); continue; }
                };
                print!("{s}");
            }
        }
    }
    println!();
}

fn le16(pair: &[u8]) -> u16 {
    if pair.len() >= 2 {
        (pair[1] as u16) << 8 | pair[0] as u16
    } else {
        pair[0] as u16
    }
}
