/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! length — print string length

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        println!("0");
        return 0;
    }
    println!("{}", args[0].len());
    0
}
