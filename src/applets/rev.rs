/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let lines = super::input_lines(args);
    for line in &lines {
        let reversed: String = line.chars().rev().collect();
        println!("{reversed}");
    }
    0
}
