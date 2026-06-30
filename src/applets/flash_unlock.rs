/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! flash_unlock — unlock MTD flash regions (see flash_lock)
use crate::applets::flash_lock;

pub fn run(args: &[String]) -> i32 {
    flash_lock::run_with(args, false)
}
