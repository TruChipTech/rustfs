/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ipaddr — show IP addresses (alias for ip addr show)

pub fn run(args: &[String]) -> i32 {
    let mut new_args = vec!["addr".to_string(), "show".to_string()];
    new_args.extend_from_slice(args);
    super::ip::run(&new_args)
}
