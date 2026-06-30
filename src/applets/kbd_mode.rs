/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! kbd_mode — get or set the keyboard translation mode.

use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

const KDGKBMODE: libc::c_ulong = 0x4B44;
const KDSKBMODE: libc::c_ulong = 0x4B45;

const K_RAW: libc::c_long = 0x00;
const K_XLATE: libc::c_long = 0x01;
const K_MEDIUMRAW: libc::c_long = 0x02;
const K_UNICODE: libc::c_long = 0x03;

pub fn run(args: &[String]) -> i32 {
    let mut set_mode: Option<libc::c_long> = None;
    let mut dev = "/dev/tty".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" => set_mode = Some(K_XLATE),
            "-k" => set_mode = Some(K_MEDIUMRAW),
            "-s" => set_mode = Some(K_RAW),
            "-u" => set_mode = Some(K_UNICODE),
            "-C" => {
                if i + 1 < args.len() {
                    dev = args[i + 1].clone();
                    i += 1;
                }
            }
            "--help" => {
                eprintln!("Usage: kbd_mode [-a|-k|-s|-u] [-C TTY]");
                return 0;
            }
            s if s.starts_with('-') => {
                eprintln!("kbd_mode: unknown option '{s}'");
                return 1;
            }
            _ => {}
        }
        i += 1;
    }

    let tty = match OpenOptions::new().read(true).write(true).open(&dev) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("kbd_mode: {dev}: {e}");
            return 1;
        }
    };
    let fd = tty.as_raw_fd();

    match set_mode {
        Some(m) => unsafe {
            if libc::ioctl(fd, KDSKBMODE as _, m) != 0 {
                eprintln!("kbd_mode: KDSKBMODE failed: {}", std::io::Error::last_os_error());
                return 1;
            }
            0
        },
        None => unsafe {
            let mut mode: libc::c_long = 0;
            if libc::ioctl(fd, KDGKBMODE as _, &mut mode) != 0 {
                eprintln!("kbd_mode: KDGKBMODE failed: {}", std::io::Error::last_os_error());
                return 1;
            }
            let name = match mode {
                K_RAW => "raw (scancode)",
                K_XLATE => "default (ASCII)",
                K_MEDIUMRAW => "mediumraw (keycode)",
                K_UNICODE => "Unicode (UTF-8)",
                _ => "unknown",
            };
            println!("The keyboard is in {name} mode");
            0
        },
    }
}
