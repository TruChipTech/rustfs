/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! comm — compare two sorted files line by line
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn run(args: &[String]) -> i32 {
    let mut s1 = false;
    let mut s2 = false;
    let mut s3 = false;
    let mut files = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 && arg != "-" {
            for c in arg[1..].chars() {
                match c {
                    '1' => s1 = true,
                    '2' => s2 = true,
                    '3' => s3 = true,
                    _ => {
                        eprintln!("comm: invalid option -- '{c}'");
                        return 1;
                    }
                }
            }
        } else {
            files.push(arg.clone());
        }
    }

    if files.len() != 2 {
        eprintln!("comm: usage: comm [-123] FILE1 FILE2");
        return 1;
    }

    let l1 = match read_lines(&files[0]) {
        Ok(v) => v,
        Err(e) => { eprintln!("comm: {}: {e}", files[0]); return 1; }
    };
    let l2 = match read_lines(&files[1]) {
        Ok(v) => v,
        Err(e) => { eprintln!("comm: {}: {e}", files[1]); return 1; }
    };

    let col2 = if s1 { "" } else { "\t" };
    let col3 = if s1 { "" } else { "\t" }.to_string() + if s2 { "" } else { "\t" };

    let (mut i, mut j) = (0, 0);
    while i < l1.len() && j < l2.len() {
        if l1[i] < l2[j] {
            if !s1 { println!("{}", l1[i]); }
            i += 1;
        } else if l1[i] > l2[j] {
            if !s2 { println!("{}{}", col2, l2[j]); }
            j += 1;
        } else {
            if !s3 { println!("{}{}", col3, l1[i]); }
            i += 1;
            j += 1;
        }
    }
    while i < l1.len() {
        if !s1 { println!("{}", l1[i]); }
        i += 1;
    }
    while j < l2.len() {
        if !s2 { println!("{}{}", col2, l2[j]); }
        j += 1;
    }
    0
}

fn read_lines(path: &str) -> io::Result<Vec<String>> {
    let reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(path)?))
    };
    reader.lines().collect()
}
