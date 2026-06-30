/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! uuencode — encode a file for safe text transmission (uu or base64)
use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs::File;
use std::io::{self, Read};

pub fn run(args: &[String]) -> i32 {
    let mut base64_mode = false;
    let mut positional = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-m" | "--base64" => base64_mode = true,
            _ => positional.push(arg.clone()),
        }
    }

    // Forms: uuencode [infile] remotefile
    let (infile, name) = match positional.len() {
        0 => { eprintln!("uuencode: missing remote file name"); return 1; }
        1 => ("-".to_string(), positional[0].clone()),
        _ => (positional[0].clone(), positional[1].clone()),
    };

    let mut data = Vec::new();
    let res = if infile == "-" {
        io::stdin().read_to_end(&mut data)
    } else {
        File::open(&infile).and_then(|mut f| f.read_to_end(&mut data))
    };
    if let Err(e) = res {
        eprintln!("uuencode: {infile}: {e}");
        return 1;
    }

    if base64_mode {
        println!("begin-base64 644 {name}");
        let enc = STANDARD.encode(&data);
        for chunk in enc.as_bytes().chunks(76) {
            println!("{}", String::from_utf8_lossy(chunk));
        }
        println!("====");
    } else {
        println!("begin 644 {name}");
        for chunk in data.chunks(45) {
            print!("{}", (b' ' + chunk.len() as u8) as char);
            for triple in chunk.chunks(3) {
                let mut b = [0u8; 3];
                b[..triple.len()].copy_from_slice(triple);
                let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
                for shift in [18, 12, 6, 0] {
                    let v = ((n >> shift) & 0x3f) as u8;
                    print!("{}", if v == 0 { '`' } else { (b' ' + v) as char });
                }
            }
            println!();
        }
        println!("`");
        println!("end");
    }
    0
}
