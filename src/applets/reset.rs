/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! reset — reset the terminal to sane defaults.

use std::io::{self, Write};

pub fn run(_args: &[String]) -> i32 {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    // ESC c  : full reset (RIS)
    // ESC ( B : select ASCII charset
    // ESC [ m : reset attributes
    // ESC [ 2J: clear screen, ESC [ H: cursor home
    let seq = b"\x1bc\x1b(B\x1b[m\x1b[2J\x1b[H";
    let _ = out.write_all(seq);
    let _ = out.flush();
    0
}
