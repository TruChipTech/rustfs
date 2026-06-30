/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! runlevel — print previous and current SysV runlevel from the utmp database.

use std::ffi::CString;

const RUN_LVL: libc::c_short = 1;

pub fn run(args: &[String]) -> i32 {
    let path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned();

    unsafe {
        if let Some(p) = &path {
            if let Ok(c) = CString::new(p.as_str()) {
                libc::utmpxname(c.as_ptr());
            }
        }
        libc::setutxent();
        let mut found = None;
        loop {
            let ent = libc::getutxent();
            if ent.is_null() {
                break;
            }
            if (*ent).ut_type == RUN_LVL {
                // For a RUN_LVL record the runlevel chars are packed into ut_pid:
                // low byte = current, high byte = previous.
                let pid = (*ent).ut_pid;
                let cur = (pid & 0xff) as u8;
                let prev = ((pid >> 8) & 0xff) as u8;
                found = Some((prev, cur));
            }
        }
        libc::endutxent();

        match found {
            Some((prev, cur)) => {
                let prev_c = if prev == 0 { 'N' } else { prev as char };
                println!("{} {}", prev_c, cur as char);
                0
            }
            None => {
                println!("unknown");
                1
            }
        }
    }
}
