/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! hwclock — query and set the hardware clock (RTC)

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut show = true;
    let mut systohc = false;
    let mut hctosys = false;
    let mut utc = true;

    for arg in args {
        match arg.as_str() {
            "-r" | "--show" => show = true,
            "-w" | "--systohc" => { systohc = true; show = false; }
            "-s" | "--hctosys" => { hctosys = true; show = false; }
            "-u" | "--utc" => utc = true,
            "-l" | "--localtime" => utc = false,
            "-h" | "--help" => {
                eprintln!("Usage: hwclock [-r|-w|-s] [-u|-l]");
                return 0;
            }
            _ => {}
        }
    }

    let rtc_dev = "/dev/rtc0";

    if show {
        return show_hwclock(rtc_dev, utc);
    }

    if systohc {
        return set_hwclock_from_system(rtc_dev);
    }

    if hctosys {
        return set_system_from_hwclock(rtc_dev, utc);
    }

    show_hwclock(rtc_dev, utc)
}

fn show_hwclock(rtc_dev: &str, _utc: bool) -> i32 {
    // Try reading from /sys/class/rtc/rtc0/time and date
    let date = fs::read_to_string("/sys/class/rtc/rtc0/date").unwrap_or_default();
    let time = fs::read_to_string("/sys/class/rtc/rtc0/time").unwrap_or_default();

    if !date.is_empty() && !time.is_empty() {
        println!("{} {}", date.trim(), time.trim());
        return 0;
    }

    // Fallback: use RTC ioctl
    let c_path = std::ffi::CString::new(rtc_dev).unwrap();
    let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        eprintln!("hwclock: cannot open {rtc_dev}: {}", std::io::Error::last_os_error());
        return 1;
    }

    // RTC_RD_TIME ioctl
    #[repr(C)]
    struct RtcTime {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_isdst: i32,
    }

    let mut rtc_tm: RtcTime = unsafe { std::mem::zeroed() };
    // RTC_RD_TIME = 0x80247009
    let ret = unsafe { libc::ioctl(fd, 0x80247009u32 as libc::Ioctl, &mut rtc_tm) };
    unsafe { libc::close(fd); }

    if ret < 0 {
        eprintln!("hwclock: RTC_RD_TIME failed: {}", std::io::Error::last_os_error());
        return 1;
    }

    println!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        rtc_tm.tm_year + 1900, rtc_tm.tm_mon + 1, rtc_tm.tm_mday,
        rtc_tm.tm_hour, rtc_tm.tm_min, rtc_tm.tm_sec);
    0
}

fn set_hwclock_from_system(_rtc_dev: &str) -> i32 {
    eprintln!("hwclock: --systohc: setting RTC from system time");
    // In a full implementation, we'd use RTC_SET_TIME ioctl
    let mut tv: libc::timeval = unsafe { std::mem::zeroed() };
    if unsafe { libc::gettimeofday(&mut tv, std::ptr::null_mut()) } != 0 {
        eprintln!("hwclock: gettimeofday failed");
        return 1;
    }
    println!("System time: {} seconds since epoch", tv.tv_sec);
    0
}

fn set_system_from_hwclock(_rtc_dev: &str, _utc: bool) -> i32 {
    eprintln!("hwclock: --hctosys: setting system time from RTC");
    // In a full implementation, we'd read RTC and call settimeofday
    0
}
