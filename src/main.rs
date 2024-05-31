use clap::Parser;
use core::panic;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// FILE to be searched.
    #[arg(short, long, value_name = "FILE")]
    file: Option<String>,

    /// Use PATTERN as the pattern.
    #[arg(short, long, value_name = "PATTERN")]
    pattern: Option<String>,

    /// Print NUM lines of trailing context after matching lines.
    #[arg(short, long, value_name = "NUM")]
    after_context: Option<usize>,

    /// Print NUM lines of leading context before matching lines.
    #[arg(short, long, value_name = "NUM")]
    before_context: Option<usize>,
}

fn main() {
    let args = Args::parse();
    let after_context = args.after_context.unwrap_or_default();
    let before_context = args.before_context.unwrap_or_default();
    let pattern = args.pattern.unwrap_or_default();
    let file = match args.file {
        Some(file) => file,
        None => panic!("File name must be provided"),
    };

    let re = match Regex::new(pattern.as_str()) {
        Ok(re) => re,
        Err(e) => panic!("{e}"),
    };

    let f = match File::open(file.as_str()) {
        Ok(f) => f,
        Err(e) => panic!("{e}"),
    };

    let lines: Vec<String> = BufReader::new(f).lines().map(|r| r.unwrap()).collect();
    for (i, line) in lines.iter().enumerate() {
        if !re.is_match(line.as_str()) {
            continue;
        }

        let lower_bound = i.saturating_sub(before_context);
        let upper_bound = (lines.len() - 1).min(i + after_context);

        for (i, line) in (lower_bound + 1..=upper_bound + 1)
            .zip(lines.iter().take(upper_bound + 1).skip(lower_bound))
        {
            println!("{i}: {line}");
        }
    }
}
