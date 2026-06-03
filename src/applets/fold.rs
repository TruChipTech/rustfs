/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let mut width: usize = 80;
    let mut break_words = false;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-w" | "--width" => {
                i += 1;
                if i < args.len() {
                    width = args[i].parse().unwrap_or(80);
                }
            }
            "-s" | "--spaces" => break_words = true,
            arg if arg.starts_with("-w") => {
                width = arg[2..].parse().unwrap_or(80);
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let lines = super::input_lines(&files);

    for line in &lines {
        if line.len() <= width {
            println!("{line}");
        } else if break_words {
            // Break at word boundaries
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut current = String::new();
            for word in words {
                if current.is_empty() {
                    current = word.to_string();
                } else if current.len() + 1 + word.len() <= width {
                    current.push(' ');
                    current.push_str(word);
                } else {
                    println!("{current}");
                    current = word.to_string();
                }
            }
            if !current.is_empty() {
                println!("{current}");
            }
        } else {
            // Break at exact width
            let chars: Vec<char> = line.chars().collect();
            for chunk in chars.chunks(width) {
                let s: String = chunk.iter().collect();
                println!("{s}");
            }
        }
    }

    0
}
