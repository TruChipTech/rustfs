/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let mut body_numbering = "t"; // t=non-empty, a=all, n=none
    let mut separator = "\t";
    let mut width: usize = 6;
    let mut increment: u64 = 1;
    let mut start: u64 = 1;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-b" => {
                i += 1;
                if i < args.len() {
                    body_numbering = if args[i] == "a" {
                        "a"
                    } else if args[i] == "n" {
                        "n"
                    } else {
                        "t"
                    };
                }
            }
            "-s" => {
                i += 1;
                if i < args.len() {
                    separator = &args[i];
                }
            }
            "-w" => {
                i += 1;
                if i < args.len() {
                    width = args[i].parse().unwrap_or(6);
                }
            }
            "-i" => {
                i += 1;
                if i < args.len() {
                    increment = args[i].parse().unwrap_or(1);
                }
            }
            "-v" => {
                i += 1;
                if i < args.len() {
                    start = args[i].parse().unwrap_or(1);
                }
            }
            "-ba" => body_numbering = "a",
            "-bt" => body_numbering = "t",
            "-bn" => body_numbering = "n",
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let lines = super::input_lines(&files);
    let mut line_num = start;

    for line in &lines {
        let should_number = match body_numbering {
            "a" => true,
            "n" => false,
            _ => !line.is_empty(), // "t" - non-empty lines
        };

        if should_number {
            println!("{:>width$}{separator}{line}", line_num, width = width, separator = separator);
            line_num += increment;
        } else {
            println!("{:>width$}{separator}", "", width = width, separator = separator);
        }
    }

    0
}
