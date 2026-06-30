/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! awk — pattern scanning and text processing

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};

pub fn run(args: &[String]) -> i32 {
    let mut field_sep = " ".to_string();
    let mut program = String::new();
    let mut files: Vec<String> = Vec::new();
    let mut vars: HashMap<String, String> = HashMap::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-F" => {
                i += 1;
                if i < args.len() {
                    field_sep = args[i].clone();
                }
            }
            "-v" => {
                i += 1;
                if i < args.len() {
                    if let Some((k, v)) = args[i].split_once('=') {
                        vars.insert(k.to_string(), v.to_string());
                    }
                }
            }
            s if !s.starts_with('-') || program.is_empty() && !s.starts_with('-') => {
                if program.is_empty() {
                    program = s.to_string();
                } else {
                    files.push(s.to_string());
                }
            }
            _ => {
                if program.is_empty() {
                    program = args[i].clone();
                } else {
                    files.push(args[i].clone());
                }
            }
        }
        i += 1;
    }

    if program.is_empty() {
        eprintln!("Usage: awk [-F sep] [-v var=val] 'program' [file...]");
        return 1;
    }

    let rules = match parse_program(&program) {
        Some(r) => r,
        None => {
            eprintln!("awk: failed to parse program");
            return 1;
        }
    };

    // Execute BEGIN blocks
    let mut state = AwkState {
        field_sep: field_sep.clone(),
        nr: 0,
        fnr: 0,
        nf: 0,
        fields: Vec::new(),
        line: String::new(),
        vars,
        ofs: " ".to_string(),
        ors: "\n".to_string(),
    };

    for rule in &rules {
        if rule.pattern == Pattern::Begin {
            execute_action(&rule.action, &mut state);
        }
    }

    if files.is_empty() {
        let stdin = io::stdin();
        for l in stdin.lock().lines().map_while(Result::ok) {
            process_line(&l, &rules, &mut state);
        }
    } else {
        for file in &files {
            state.fnr = 0;
            match fs::read_to_string(file) {
                Ok(content) => {
                    for line in content.lines() {
                        process_line(line, &rules, &mut state);
                    }
                }
                Err(e) => { eprintln!("awk: {file}: {e}"); return 1; }
            }
        }
    }

    // Execute END blocks
    for rule in &rules {
        if rule.pattern == Pattern::End {
            execute_action(&rule.action, &mut state);
        }
    }

    0
}

struct AwkState {
    field_sep: String,
    nr: usize,
    fnr: usize,
    nf: usize,
    fields: Vec<String>,
    line: String,
    vars: HashMap<String, String>,
    ofs: String,
    #[allow(dead_code)]
    ors: String,
}

#[derive(Debug, PartialEq)]
enum Pattern {
    Begin,
    End,
    Always,
    Regex(String),
    Expression(String),
}

struct Rule {
    pattern: Pattern,
    action: String,
}

fn parse_program(prog: &str) -> Option<Vec<Rule>> {
    let mut rules = Vec::new();
    let prog = prog.trim();

    // Simple parser: handle common patterns
    // BEGIN { ... } /pattern/ { ... } END { ... } { ... }
    let mut pos = 0;
    let bytes = prog.as_bytes();

    while pos < bytes.len() {
        // Skip whitespace
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() { pos += 1; }
        if pos >= bytes.len() { break; }

        let pattern;
        // Check for BEGIN/END
        if prog[pos..].starts_with("BEGIN") {
            pattern = Pattern::Begin;
            pos += 5;
        } else if prog[pos..].starts_with("END") {
            pattern = Pattern::End;
            pos += 3;
        } else if bytes[pos] == b'/' {
            // Regex pattern
            pos += 1;
            let start = pos;
            while pos < bytes.len() && bytes[pos] != b'/' { pos += 1; }
            let regex = prog[start..pos].to_string();
            if pos < bytes.len() { pos += 1; }
            pattern = Pattern::Regex(regex);
        } else if bytes[pos] == b'{' {
            pattern = Pattern::Always;
        } else {
            // Expression pattern (e.g., NR==1, $1 > 5)
            let start = pos;
            while pos < bytes.len() && bytes[pos] != b'{' { pos += 1; }
            let expr = prog[start..pos].trim().to_string();
            pattern = Pattern::Expression(expr);
        }

        // Skip whitespace
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() { pos += 1; }

        // Find action block
        if pos < bytes.len() && bytes[pos] == b'{' {
            pos += 1;
            let start = pos;
            let mut depth = 1;
            while pos < bytes.len() && depth > 0 {
                if bytes[pos] == b'{' { depth += 1; }
                if bytes[pos] == b'}' { depth -= 1; }
                if depth > 0 { pos += 1; }
            }
            let action = prog[start..pos].trim().to_string();
            if pos < bytes.len() { pos += 1; }
            rules.push(Rule { pattern, action });
        } else {
            // No action block, default to { print }
            rules.push(Rule { pattern, action: "print".to_string() });
            break;
        }
    }

    if rules.is_empty() {
        // Bare action like just "{ print $1 }" or just "print $1"
        rules.push(Rule {
            pattern: Pattern::Always,
            action: prog.trim_start_matches('{').trim_end_matches('}').trim().to_string(),
        });
    }

    Some(rules)
}

fn process_line(line: &str, rules: &[Rule], state: &mut AwkState) {
    state.nr += 1;
    state.fnr += 1;
    state.line = line.to_string();

    // Split fields
    if state.field_sep == " " {
        state.fields = line.split_whitespace().map(|s| s.to_string()).collect();
    } else {
        state.fields = line.split(&state.field_sep).map(|s| s.to_string()).collect();
    }
    state.nf = state.fields.len();

    for rule in rules {
        match &rule.pattern {
            Pattern::Begin | Pattern::End => continue,
            Pattern::Always => execute_action(&rule.action, state),
            Pattern::Regex(re) => {
                if line.contains(re.as_str()) {
                    execute_action(&rule.action, state);
                }
            }
            Pattern::Expression(expr) => {
                if evaluate_condition(expr, state) {
                    execute_action(&rule.action, state);
                }
            }
        }
    }
}

fn evaluate_condition(expr: &str, state: &AwkState) -> bool {
    let expr = expr.trim();
    // Simple condition evaluation
    if let Some((left, right)) = expr.split_once("==") {
        let l = resolve_value(left.trim(), state);
        let r = resolve_value(right.trim(), state);
        return l == r;
    }
    if let Some((left, right)) = expr.split_once("!=") {
        let l = resolve_value(left.trim(), state);
        let r = resolve_value(right.trim(), state);
        return l != r;
    }
    if let Some((left, right)) = expr.split_once(">=") {
        let l: f64 = resolve_value(left.trim(), state).parse().unwrap_or(0.0);
        let r: f64 = resolve_value(right.trim(), state).parse().unwrap_or(0.0);
        return l >= r;
    }
    if let Some((left, right)) = expr.split_once("<=") {
        let l: f64 = resolve_value(left.trim(), state).parse().unwrap_or(0.0);
        let r: f64 = resolve_value(right.trim(), state).parse().unwrap_or(0.0);
        return l <= r;
    }
    if let Some((left, right)) = expr.split_once('>') {
        let l: f64 = resolve_value(left.trim(), state).parse().unwrap_or(0.0);
        let r: f64 = resolve_value(right.trim(), state).parse().unwrap_or(0.0);
        return l > r;
    }
    if let Some((left, right)) = expr.split_once('<') {
        let l: f64 = resolve_value(left.trim(), state).parse().unwrap_or(0.0);
        let r: f64 = resolve_value(right.trim(), state).parse().unwrap_or(0.0);
        return l < r;
    }
    // Regex match ~/regex/
    if let Some((left, right)) = expr.split_once('~') {
        let val = resolve_value(left.trim(), state);
        let pat = right.trim().trim_matches('/');
        return val.contains(pat);
    }
    true
}

fn resolve_value(token: &str, state: &AwkState) -> String {
    if let Some(idx_str) = token.strip_prefix('$') {
        if idx_str == "0" {
            return state.line.clone();
        }
        if let Ok(idx) = idx_str.parse::<usize>() {
            if idx > 0 && idx <= state.fields.len() {
                return state.fields[idx - 1].clone();
            }
        }
        return String::new();
    }
    if token == "NR" { return state.nr.to_string(); }
    if token == "NF" { return state.nf.to_string(); }
    if token == "FNR" { return state.fnr.to_string(); }
    if let Some(v) = state.vars.get(token) {
        return v.clone();
    }
    // Strip quotes
    if (token.starts_with('"') && token.ends_with('"')) ||
       (token.starts_with('\'') && token.ends_with('\'')) {
        return token[1..token.len()-1].to_string();
    }
    token.to_string()
}

fn execute_action(action: &str, state: &mut AwkState) {
    // Simple action executor
    for stmt in action.split(';') {
        let stmt = stmt.trim();
        if stmt.is_empty() { continue; }

        if stmt == "print" || stmt == "print $0" {
            println!("{}", state.line);
        } else if let Some(rest) = stmt.strip_prefix("print ") {
            let parts: Vec<&str> = rest.split(',').collect();
            let mut output = Vec::new();
            for part in parts {
                let val = resolve_value(part.trim(), state);
                output.push(val);
            }
            println!("{}", output.join(&state.ofs));
        } else if let Some(rest) = stmt.strip_prefix("printf ") {
            // Basic printf support
            let resolved = resolve_value(rest.trim(), state);
            print!("{}", resolved.replace("\\n", "\n").replace("\\t", "\t"));
        }
    }
}
