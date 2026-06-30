/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! adjtimex — read or set kernel time variables

pub fn run(args: &[String]) -> i32 {
    let mut buf: libc::timex = unsafe { std::mem::zeroed() };
    let mut modes: u32 = 0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--print" => {} // print is the default behaviour
            "-q" | "--quiet" => {}
            "-o" | "--offset" => { i += 1; if let Some(v) = args.get(i).and_then(|s| s.parse().ok()) { buf.offset = v; modes |= libc::ADJ_OFFSET; } }
            "-f" | "--frequency" => { i += 1; if let Some(v) = args.get(i).and_then(|s| s.parse().ok()) { buf.freq = v; modes |= libc::ADJ_FREQUENCY; } }
            "-t" | "--tick" => { i += 1; if let Some(v) = args.get(i).and_then(|s| s.parse().ok()) { buf.tick = v; modes |= libc::ADJ_TICK; } }
            _ => {}
        }
        i += 1;
    }

    buf.modes = modes as _;
    let ret = unsafe { libc::adjtimex(&mut buf) };
    if ret < 0 {
        eprintln!("adjtimex: {}", std::io::Error::last_os_error());
        return 1;
    }

    println!("    mode:         {}", buf.modes);
    println!("-o  offset:       {}", buf.offset);
    println!("-f  frequency:    {}", buf.freq);
    println!("    maxerror:     {}", buf.maxerror);
    println!("    esterror:     {}", buf.esterror);
    println!("    status:       {}", buf.status);
    println!("-t  tick:         {}", buf.tick);
    let state = match ret {
        0 => "TIME_OK",
        1 => "TIME_INS",
        2 => "TIME_DEL",
        3 => "TIME_OOP",
        4 => "TIME_WAIT",
        5 => "TIME_ERROR",
        _ => "unknown",
    };
    println!("    return value: {ret} ({state})");
    0
}
