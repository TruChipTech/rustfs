/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let text = if args.is_empty() {
        "y"
    } else {
        &args.join(" ")
    };

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let bytes = format!("{text}\n");

    loop {
        if out.write_all(bytes.as_bytes()).is_err() {
            break;
        }
    }

    0
}
