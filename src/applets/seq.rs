/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let mut first: f64 = 1.0;
    let mut increment: f64 = 1.0;
    let last: f64;
    let mut separator = "\n";
    let mut format_str: Option<String> = None;

    let mut positional = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--separator" => {
                i += 1;
                if i < args.len() {
                    separator = &args[i];
                }
            }
            "-f" | "--format" => {
                i += 1;
                if i < args.len() {
                    format_str = Some(args[i].clone());
                }
            }
            _ => positional.push(args[i].clone()),
        }
        i += 1;
    }

    match positional.len() {
        1 => {
            last = positional[0].parse().unwrap_or(0.0);
        }
        2 => {
            first = positional[0].parse().unwrap_or(1.0);
            last = positional[1].parse().unwrap_or(0.0);
        }
        3 => {
            first = positional[0].parse().unwrap_or(1.0);
            increment = positional[1].parse().unwrap_or(1.0);
            last = positional[2].parse().unwrap_or(0.0);
        }
        _ => {
            eprintln!("seq: missing operand");
            return 1;
        }
    }

    // Handle negative increments properly
    if increment == 0.0 {
        eprintln!("seq: increment must not be zero");
        return 1;
    }

    // Determine decimal places from input
    let decimal_places = positional
        .iter()
        .map(|s| {
            if let Some(dot_pos) = s.find('.') {
                s.len() - dot_pos - 1
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0);

    let mut current = first;
    let mut is_first = true;

    loop {
        if increment > 0.0 && current > last {
            break;
        }
        if increment < 0.0 && current < last {
            break;
        }

        if !is_first {
            print!("{separator}");
        }
        is_first = false;

        if let Some(ref fmt) = format_str {
            // Simple format handling
            print!("{}", fmt.replace("%g", &format!("{current}")));
        } else if decimal_places > 0 {
            print!("{current:.prec$}", prec = decimal_places);
        } else {
            print!("{}", current as i64);
        }

        current += increment;
    }

    if !is_first {
        println!();
    }

    0
}
