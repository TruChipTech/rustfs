/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};

pub fn run(args: &[String]) -> i32 {
    let mut append = false;
    let mut files = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-a" | "--append" => append = true,
            _ => files.push(arg.clone()),
        }
    }

    let mut outputs: Vec<Box<dyn Write>> = Vec::new();

    // Always write to stdout
    outputs.push(Box::new(io::stdout()));

    // Open output files
    for file in &files {
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(file);

        match f {
            Ok(f) => outputs.push(Box::new(f)),
            Err(e) => {
                eprintln!("tee: {file}: {e}");
            }
        }
    }

    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        match line {
            Ok(l) => {
                let bytes = format!("{l}\n");
                for out in &mut outputs {
                    let _ = out.write_all(bytes.as_bytes());
                }
            }
            Err(_) => break,
        }
    }

    for out in &mut outputs {
        let _ = out.flush();
    }

    0
}
