/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(_args: &[String]) -> i32 {
    unsafe { libc::sync() };
    0
}
