/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! cal — display a calendar
use chrono::{Datelike, Local};

pub fn run(args: &[String]) -> i32 {
    let nums: Vec<i32> = args.iter()
        .filter(|a| !a.starts_with('-'))
        .filter_map(|a| a.parse().ok())
        .collect();

    let now = Local::now();
    let (month, year) = match nums.len() {
        0 => (now.month() as i32, now.year()),
        1 => {
            // single arg is a year -> print whole year would be large; busybox
            // treats single numeric arg as year. Print current month of that year.
            (now.month() as i32, nums[0])
        }
        _ => (nums[0], nums[1]),
    };

    if !(1..=12).contains(&month) {
        eprintln!("cal: invalid month");
        return 1;
    }

    print_month(month as u32, year);
    0
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(m: u32, y: i32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap(y) { 29 } else { 28 },
        _ => 30,
    }
}

/// Zeller-based day of week for the 1st (0 = Sunday).
fn first_weekday(m: u32, y: i32) -> u32 {
    let (mut mm, mut yy) = (m as i32, y);
    if mm < 3 { mm += 12; yy -= 1; }
    let k = yy % 100;
    let j = yy / 100;
    let h = (1 + 13 * (mm + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    // Zeller h: 0=Saturday. Convert to 0=Sunday.
    ((h + 6) % 7) as u32
}

fn print_month(m: u32, y: i32) {
    let names = ["", "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December"];
    let title = format!("{} {}", names[m as usize], y);
    let width = 20;
    let pad = (width - title.len()) / 2;
    println!("{:pad$}{}", "", title, pad = pad);
    println!("Su Mo Tu We Th Fr Sa");

    let start = first_weekday(m, y);
    let dim = days_in_month(m, y);
    let mut col = 0;
    for _ in 0..start {
        print!("   ");
        col += 1;
    }
    for d in 1..=dim {
        print!("{d:2} ");
        col += 1;
        if col % 7 == 0 {
            println!();
        }
    }
    if col % 7 != 0 {
        println!();
    }
}
