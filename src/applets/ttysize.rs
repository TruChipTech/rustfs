/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ttysize — print the terminal width and height

pub fn run(args: &[String]) -> i32 {
    let (mut w, mut h) = winsize();
    // Optional args 'w'/'h' select what to print and in which order.
    if args.is_empty() {
        println!("{w} {h}");
        return 0;
    }
    let mut out = Vec::new();
    for a in args {
        match a.as_str() {
            "w" => out.push(w.to_string()),
            "h" => out.push(h.to_string()),
            _ => {}
        }
    }
    let _ = (&mut w, &mut h);
    println!("{}", out.join(" "));
    0
}

fn winsize() -> (u16, u16) {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    for fd in 0..3 {
        if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) } == 0 && ws.ws_col != 0 {
            return (ws.ws_col, ws.ws_row);
        }
    }
    (80, 24)
}
