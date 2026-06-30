/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! fdformat — low-level format a floppy disk
use std::os::unix::io::AsRawFd;

const FDFMTBEG: libc::c_ulong = 0x0247;
const FDFMTTRK: libc::c_ulong = 0x0248;
const FDFMTEND: libc::c_ulong = 0x0249;
const FDGETPRM: libc::c_ulong = 0x0204;

// struct floppy_struct — only the leading fields we need.
#[repr(C)]
#[derive(Default)]
struct FloppyStruct {
    size: libc::c_uint,
    sect: libc::c_uint,
    head: libc::c_uint,
    track: libc::c_uint,
    stretch: libc::c_uint,
    // remaining fields unused
    gap: u8,
    rate: u8,
    spec1: u8,
    fmt_gap: u8,
}

// struct format_descr { unsigned device, head, track; }
#[repr(C)]
struct FormatDescr {
    device: libc::c_uint,
    head: libc::c_uint,
    track: libc::c_uint,
}

pub fn run(args: &[String]) -> i32 {
    let mut verify = true;
    let mut dev = None;
    for a in args {
        match a.as_str() {
            "-n" => verify = false,
            s if !s.starts_with('-') => dev = Some(s.to_string()),
            _ => {}
        }
    }
    let dev = match dev { Some(d) => d, None => { eprintln!("Usage: fdformat [-n] DEVICE"); return 1; } };

    let file = match std::fs::OpenOptions::new().read(true).write(true).open(&dev) {
        Ok(f) => f,
        Err(e) => { eprintln!("fdformat: {dev}: {e}"); return 1; }
    };
    let fd = file.as_raw_fd();

    let mut prm = FloppyStruct::default();
    if unsafe { libc::ioctl(fd, FDGETPRM as _, &mut prm) } != 0 {
        eprintln!("fdformat: {dev}: cannot get floppy parameters: {}", std::io::Error::last_os_error());
        return 1;
    }

    println!("Formatting {} tracks, {} sectors/track, {} heads",
        prm.track, prm.sect, prm.head);

    if unsafe { libc::ioctl(fd, FDFMTBEG as _) } != 0 {
        eprintln!("fdformat: FDFMTBEG: {}", std::io::Error::last_os_error());
        return 1;
    }

    for track in 0..prm.track {
        for head in 0..prm.head {
            let descr = FormatDescr { device: 0, head, track };
            if unsafe { libc::ioctl(fd, FDFMTTRK as _, &descr) } != 0 {
                eprintln!("fdformat: track {track} head {head}: {}", std::io::Error::last_os_error());
                unsafe { libc::ioctl(fd, FDFMTEND as _); }
                return 1;
            }
        }
        print!("\rformatting track {track} ");
    }
    println!();

    if unsafe { libc::ioctl(fd, FDFMTEND as _) } != 0 {
        eprintln!("fdformat: FDFMTEND: {}", std::io::Error::last_os_error());
        return 1;
    }

    if verify {
        // Read every sector back to verify the format took.
        use std::io::Read;
        let mut f = file;
        let total = (prm.size as usize) * 512;
        let mut buf = vec![0u8; 512];
        let mut read = 0;
        while read < total {
            match f.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => read += n,
                Err(e) => { eprintln!("fdformat: verify failed at {read}: {e}"); return 1; }
            }
        }
        println!("Verifying ... done");
    }
    0
}
