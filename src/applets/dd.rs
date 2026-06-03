/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! dd — convert and copy a file

use std::fs;
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::time::Instant;

pub fn run(args: &[String]) -> i32 {
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut bs: usize = 512;
    let mut ibs: usize = 0;
    let mut obs: usize = 0;
    let mut count: Option<usize> = None;
    let mut skip: usize = 0;
    let mut seek: usize = 0;
    let mut conv_flags: Vec<String> = Vec::new();

    for arg in args {
        if let Some((key, val)) = arg.split_once('=') {
            match key {
                "if" => input = Some(val.to_string()),
                "of" => output = Some(val.to_string()),
                "bs" => bs = parse_size(val),
                "ibs" => ibs = parse_size(val),
                "obs" => obs = parse_size(val),
                "count" => count = val.parse().ok(),
                "skip" => skip = val.parse().unwrap_or(0),
                "seek" => seek = val.parse().unwrap_or(0),
                "conv" => conv_flags = val.split(',').map(|s| s.to_string()).collect(),
                _ => {
                    eprintln!("dd: unknown operand: {key}");
                    return 1;
                }
            }
        }
    }

    if ibs == 0 { ibs = bs; }
    if obs == 0 { obs = bs; }

    let start = Instant::now();

    // Open input
    let mut reader: Box<dyn Read> = if let Some(ref path) = input {
        match fs::File::open(path) {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("dd: failed to open '{path}': {e}");
                return 1;
            }
        }
    } else {
        Box::new(io::stdin())
    };

    // Open output
    let mut writer: Box<dyn Write> = if let Some(ref path) = output {
        let f = if conv_flags.contains(&"notrunc".to_string()) {
            fs::OpenOptions::new().write(true).open(path)
        } else {
            fs::File::create(path)
        };
        match f {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("dd: failed to open '{path}': {e}");
                return 1;
            }
        }
    } else {
        Box::new(io::stdout())
    };

    // Skip input blocks
    if skip > 0 {
        let skip_bytes = skip * ibs;
        if let Some(ref path) = input {
            if let Ok(mut f) = fs::File::open(path) {
                let _ = f.seek(SeekFrom::Start(skip_bytes as u64));
                reader = Box::new(f);
            }
        } else {
            let mut discard = vec![0u8; ibs];
            for _ in 0..skip {
                if reader.read(&mut discard).unwrap_or(0) == 0 { break; }
            }
        }
    }

    // Seek output blocks
    if seek > 0 {
        if let Some(ref path) = output {
            if let Ok(mut f) = fs::OpenOptions::new().write(true).open(path) {
                let _ = f.seek(SeekFrom::Start((seek * obs) as u64));
                writer = Box::new(f);
            }
        }
    }

    let noerror = conv_flags.contains(&"noerror".to_string());
    let sync_pad = conv_flags.contains(&"sync".to_string());
    let swab = conv_flags.contains(&"swab".to_string());
    let ucase = conv_flags.contains(&"ucase".to_string());
    let lcase = conv_flags.contains(&"lcase".to_string());

    let mut buf = vec![0u8; ibs];
    let mut records_in = 0usize;
    let mut records_out = 0usize;
    let mut partial_in = 0usize;
    let mut partial_out = 0usize;
    let mut total_bytes = 0u64;
    let mut block_num = 0usize;

    loop {
        if let Some(c) = count {
            if block_num >= c { break; }
        }
        block_num += 1;

        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                if noerror {
                    eprintln!("dd: read error: {e}");
                    continue;
                } else {
                    eprintln!("dd: read error: {e}");
                    return 1;
                }
            }
        };

        if n == ibs {
            records_in += 1;
        } else {
            partial_in += 1;
        }

        let mut data = if sync_pad && n < ibs {
            let mut padded = buf[..n].to_vec();
            padded.resize(ibs, 0);
            padded
        } else {
            buf[..n].to_vec()
        };

        // Apply conversions
        if swab {
            for chunk in data.chunks_mut(2) {
                if chunk.len() == 2 {
                    chunk.swap(0, 1);
                }
            }
        }
        if ucase {
            for b in &mut data {
                if b.is_ascii_lowercase() { *b = b.to_ascii_uppercase(); }
            }
        }
        if lcase {
            for b in &mut data {
                if b.is_ascii_uppercase() { *b = b.to_ascii_lowercase(); }
            }
        }

        match writer.write_all(&data) {
            Ok(()) => {
                total_bytes += data.len() as u64;
                if data.len() == obs {
                    records_out += 1;
                } else {
                    partial_out += 1;
                }
            }
            Err(e) => {
                eprintln!("dd: write error: {e}");
                return 1;
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 { total_bytes as f64 / elapsed } else { 0.0 };

    eprintln!("{records_in}+{partial_in} records in");
    eprintln!("{records_out}+{partial_out} records out");
    eprintln!("{total_bytes} bytes ({:.1} kB) copied, {elapsed:.6} s, {:.1} kB/s",
        total_bytes as f64 / 1000.0, speed / 1000.0);

    0
}

fn parse_size(s: &str) -> usize {
    let s = s.trim();
    if let Some(rest) = s.strip_suffix('K') {
        return rest.parse::<usize>().unwrap_or(512) * 1024;
    }
    if let Some(rest) = s.strip_suffix('M') {
        return rest.parse::<usize>().unwrap_or(512) * 1024 * 1024;
    }
    if let Some(rest) = s.strip_suffix('G') {
        return rest.parse::<usize>().unwrap_or(512) * 1024 * 1024 * 1024;
    }
    if let Some(rest) = s.strip_suffix('k') {
        return rest.parse::<usize>().unwrap_or(512) * 1000;
    }
    s.parse().unwrap_or(512)
}
