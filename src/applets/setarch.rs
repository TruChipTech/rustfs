/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! setarch / linux32 / linux64 — run a program with a modified execution domain.

use std::os::unix::process::CommandExt;
use std::process::Command;

const PER_LINUX: libc::c_ulong = 0x0000;
const PER_LINUX32: libc::c_ulong = 0x0008;
const UNAME26: libc::c_ulong = 0x0020000;
const ADDR_LIMIT_32BIT: libc::c_ulong = 0x0800000;
const ADDR_NO_RANDOMIZE: libc::c_ulong = 0x0040000;

/// Generic entry: `setarch [ARCH] [options] PROGRAM [ARGS...]`.
pub fn run(args: &[String]) -> i32 {
    run_inner(args, None)
}

/// `linux32` — force a 32-bit personality.
pub fn run_linux32(args: &[String]) -> i32 {
    run_inner(args, Some(PER_LINUX32))
}

/// `linux64` — force a 64-bit (default) personality.
pub fn run_linux64(args: &[String]) -> i32 {
    run_inner(args, Some(PER_LINUX))
}

fn run_inner(args: &[String], forced: Option<libc::c_ulong>) -> i32 {
    let mut persona = forced.unwrap_or(PER_LINUX);
    let mut i = 0;

    // When invoked as `setarch`, the first non-option token is the arch name.
    if forced.is_none() {
        if let Some(arch) = args.first() {
            if !arch.starts_with('-') {
                if is_32bit_arch(arch) {
                    persona = PER_LINUX32;
                }
                i = 1;
            }
        }
    }

    while i < args.len() {
        match args[i].as_str() {
            "-3" | "--uname-2.6" => persona |= UNAME26,
            "-L" | "--addr-compat-layout" => persona |= ADDR_LIMIT_32BIT,
            "-R" | "--addr-no-randomize" => persona |= ADDR_NO_RANDOMIZE,
            "--help" => {
                eprintln!("Usage: setarch [ARCH] [-3] [-L] [-R] PROGRAM [ARGS...]");
                return 0;
            }
            "--" => {
                i += 1;
                break;
            }
            s if s.starts_with('-') => {
                eprintln!("setarch: unknown option '{s}'");
                return 1;
            }
            _ => break,
        }
        i += 1;
    }

    if i >= args.len() {
        eprintln!("setarch: missing PROGRAM");
        return 1;
    }

    let program = &args[i];
    let rest = &args[i + 1..];

    let mut cmd = Command::new(program);
    cmd.args(rest);
    unsafe {
        cmd.pre_exec(move || {
            if libc::personality(persona) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let err = cmd.exec(); // only returns on failure
    eprintln!("setarch: {program}: {err}");
    127
}

fn is_32bit_arch(arch: &str) -> bool {
    matches!(
        arch,
        "linux32" | "i386" | "i486" | "i586" | "i686" | "x86" | "arm" | "armv7l" | "ppc" | "mips"
    )
}
