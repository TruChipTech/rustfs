/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! dc — an arbitrary precision reverse-polish desk calculator (subset)
use std::io::{self, Read};

pub fn run(args: &[String]) -> i32 {
    let mut stack: Vec<f64> = Vec::new();
    let mut program = String::new();

    let mut i = 0;
    let mut had_e = false;
    while i < args.len() {
        match args[i].as_str() {
            "-e" | "--expression" => {
                i += 1;
                if let Some(e) = args.get(i) { program.push_str(e); program.push(' '); }
                had_e = true;
            }
            "-f" | "--file" => {
                i += 1;
                if let Some(f) = args.get(i) {
                    if let Ok(c) = std::fs::read_to_string(f) { program.push_str(&c); }
                }
                had_e = true;
            }
            s => { program.push_str(s); program.push(' '); had_e = true; }
        }
        i += 1;
    }

    if !had_e {
        let mut s = String::new();
        let _ = io::stdin().read_to_string(&mut s);
        program.push_str(&s);
    }

    for tok in program.split_whitespace() {
        if let Ok(n) = tok.parse::<f64>() {
            stack.push(n);
            continue;
        }
        match tok {
            "+" | "-" | "*" | "/" | "%" | "^" => {
                let b = stack.pop().unwrap_or(0.0);
                let a = stack.pop().unwrap_or(0.0);
                let r = match tok {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => if b != 0.0 { a / b } else { eprintln!("dc: divide by zero"); a },
                    "%" => if b != 0.0 { a % b } else { eprintln!("dc: divide by zero"); a },
                    "^" => a.powf(b),
                    _ => unreachable!(),
                };
                stack.push(r);
            }
            "p" => { if let Some(v) = stack.last() { println!("{}", fmt(*v)); } }
            "n" => { if let Some(v) = stack.pop() { print!("{}", fmt(v)); } }
            "f" => { for v in &stack { println!("{}", fmt(*v)); } }
            "c" => stack.clear(),
            "d" => { if let Some(v) = stack.last().copied() { stack.push(v); } }
            "r" => { let l = stack.len(); if l >= 2 { stack.swap(l - 1, l - 2); } }
            "" => {}
            _ => eprintln!("dc: {tok}: unimplemented"),
        }
    }
    0
}

fn fmt(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}
