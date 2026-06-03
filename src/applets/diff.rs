/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! diff — compare files line by line

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut unified = false;
    let mut context_lines: usize = 3;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-u" | "--unified" => unified = true,
            "-U" => {
                unified = true;
                i += 1;
                if i < args.len() { context_lines = args[i].parse().unwrap_or(3); }
            }
            "-q" | "--brief" => {}
            "-h" | "--help" => {
                eprintln!("Usage: diff [-u] [-U NUM] FILE1 FILE2");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    if files.len() != 2 {
        eprintln!("Usage: diff [-u] FILE1 FILE2");
        return 1;
    }

    let content1 = match fs::read_to_string(&files[0]) {
        Ok(c) => c,
        Err(e) => { eprintln!("diff: {}: {e}", files[0]); return 2; }
    };

    let content2 = match fs::read_to_string(&files[1]) {
        Ok(c) => c,
        Err(e) => { eprintln!("diff: {}: {e}", files[1]); return 2; }
    };

    let lines1: Vec<&str> = content1.lines().collect();
    let lines2: Vec<&str> = content2.lines().collect();

    // Compute LCS-based diff
    let edits = compute_diff(&lines1, &lines2);

    if edits.is_empty() {
        return 0; // Files are identical
    }

    if unified {
        print_unified_diff(&files[0], &files[1], &lines1, &lines2, &edits, context_lines);
    } else {
        print_normal_diff(&edits);
    }

    1
}

#[derive(Debug, Clone)]
enum Edit {
    Equal(usize, usize, String),   // line_a, line_b, text
    Delete(usize, String),          // line_a, text
    Insert(usize, String),          // line_b, text
}

fn compute_diff(a: &[&str], b: &[&str]) -> Vec<Edit> {
    let n = a.len();
    let m = b.len();

    // Build LCS table
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack
    let mut edits = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && a[i - 1] == b[j - 1] {
            edits.push(Edit::Equal(i, j, a[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            edits.push(Edit::Insert(j, b[j - 1].to_string()));
            j -= 1;
        } else if i > 0 {
            edits.push(Edit::Delete(i, a[i - 1].to_string()));
            i -= 1;
        }
    }

    edits.reverse();

    // Filter out equal-only results
    let has_changes = edits.iter().any(|e| !matches!(e, Edit::Equal(..)));
    if !has_changes {
        return Vec::new();
    }

    edits
}

fn print_normal_diff(edits: &[Edit]) {
    for edit in edits {
        match edit {
            Edit::Delete(line, text) => println!("{line}d\n< {text}"),
            Edit::Insert(line, text) => println!("{line}a\n> {text}"),
            Edit::Equal(..) => {}
        }
    }
}

fn print_unified_diff(
    file1: &str, file2: &str,
    _lines1: &[&str], _lines2: &[&str],
    edits: &[Edit], context: usize,
) {
    println!("--- {file1}");
    println!("+++ {file2}");

    // Group edits into hunks
    let mut hunk_start = true;
    let mut context_count = 0;
    let mut hunk_a_start = 0;
    let mut hunk_b_start = 0;

    for edit in edits {
        match edit {
            Edit::Equal(la, lb, text) => {
                if hunk_start {
                    context_count += 1;
                    if context_count <= context {
                        if hunk_a_start == 0 { hunk_a_start = *la; hunk_b_start = *lb; }
                        println!(" {text}");
                    }
                } else {
                    context_count += 1;
                    if context_count <= context {
                        println!(" {text}");
                    }
                }
            }
            Edit::Delete(la, text) => {
                if hunk_start || context_count > 0 {
                    if hunk_a_start == 0 { hunk_a_start = *la; hunk_b_start = 0; }
                    println!("@@ -{hunk_a_start} +{hunk_b_start} @@");
                    hunk_start = false;
                }
                context_count = 0;
                println!("-{text}");
            }
            Edit::Insert(lb, text) => {
                if hunk_start || context_count > 0 {
                    if hunk_b_start == 0 { hunk_b_start = *lb; }
                    if hunk_start {
                        println!("@@ -{hunk_a_start} +{hunk_b_start} @@");
                        hunk_start = false;
                    }
                }
                context_count = 0;
                println!("+{text}");
            }
        }
    }
}
