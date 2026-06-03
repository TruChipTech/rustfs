/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! hostid — print the numeric host identifier

pub fn run(_args: &[String]) -> i32 {
    let id = unsafe { libc::gethostid() };
    println!("{:08x}", id as u32);
    0
}
