/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! clear — clear the terminal screen

pub fn run(_args: &[String]) -> i32 {
    // ANSI escape: clear screen and move cursor to home
    print!("\x1B[2J\x1B[H");
    0
}
