/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! hdparm — get/set hard disk parameters (common subset)
use std::os::unix::io::AsRawFd;

const HDIO_GETGEO: libc::c_ulong = 0x0301;
const BLKGETSIZE64: libc::c_ulong = 0x80081272;
const BLKSSZGET: libc::c_ulong = 0x1268;
const BLKROGET: libc::c_ulong = 0x125e;
const BLKRRPART: libc::c_ulong = 0x125f;

#[repr(C)]
#[derive(Default)]
struct HdGeometry {
    heads: u8,
    sectors: u8,
    cylinders: u16,
    start: libc::c_ulong,
}

pub fn run(args: &[String]) -> i32 {
    let mut geo = false;
    let mut getsize = false;
    let mut readonly = false;
    let mut reread = false;
    let mut devices: Vec<String> = Vec::new();

    for a in args {
        match a.as_str() {
            "-g" => geo = true,
            "--getsz" | "--getsize64" => getsize = true,
            "-r" => readonly = true,
            "-z" => reread = true,
            s if !s.starts_with('-') => devices.push(s.to_string()),
            _ => {}
        }
    }

    if devices.is_empty() {
        eprintln!("Usage: hdparm [-g] [--getsz] [-r] [-z] DEVICE...");
        return 1;
    }

    let mut rc = 0;
    for dev in &devices {
        let file = match std::fs::File::open(dev) {
            Ok(f) => f,
            Err(e) => { eprintln!("hdparm: {dev}: {e}"); rc = 1; continue; }
        };
        let fd = file.as_raw_fd();
        println!("\n{dev}:");

        if reread
            && unsafe { libc::ioctl(fd, BLKRRPART as _) } == 0 {
                println!(" re-reading partition table");
            }
        if geo || (!getsize && !readonly && !reread) {
            let mut g = HdGeometry::default();
            if unsafe { libc::ioctl(fd, HDIO_GETGEO as _, &mut g) } == 0 {
                println!(" geometry      = {}/{}/{}, start = {}",
                    g.cylinders, g.heads, g.sectors, g.start);
            }
            let mut ssz: libc::c_int = 0;
            if unsafe { libc::ioctl(fd, BLKSSZGET as _, &mut ssz) } == 0 {
                println!(" sector size   = {ssz} bytes");
            }
            let mut bytes: u64 = 0;
            if unsafe { libc::ioctl(fd, BLKGETSIZE64 as _, &mut bytes) } == 0 {
                println!(" device size   = {} bytes ({} MiB)", bytes, bytes / (1024 * 1024));
            }
        }
        if getsize {
            let mut bytes: u64 = 0;
            if unsafe { libc::ioctl(fd, BLKGETSIZE64 as _, &mut bytes) } == 0 {
                println!(" {}", bytes / 512);
            }
        }
        if readonly {
            let mut ro: libc::c_int = 0;
            if unsafe { libc::ioctl(fd, BLKROGET as _, &mut ro) } == 0 {
                println!(" readonly      = {}", if ro != 0 { "on" } else { "off" });
            }
        }
    }
    rc
}
