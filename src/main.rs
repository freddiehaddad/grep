use clap::Parser;
use interval::{Interval, IntervalError};
use itertools::Itertools;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::exit;
use std::thread;

mod interval;

fn find_matching_lines(lines: &[String], regex: &Regex) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| match regex.is_match(line) {
            true => Some(i),
            false => None,
        })
        .collect() // turns anything iterable into a collection
}

fn create_intervals(
    lines: Vec<usize>,
    before_context: usize,
    after_context: usize,
) -> Result<Vec<Interval<usize>>, IntervalError> {
    lines
        .iter()
        .map(|line| {
            let start = line.saturating_sub(before_context);
            let end = line.saturating_add(after_context);
            Interval::new(start, end)
        })
        .collect()
}

fn merge_intervals(intervals: Vec<Interval<usize>>) -> Vec<Interval<usize>> {
    // merge overlapping intervals
    intervals
        .into_iter()
        .coalesce(|p, c| p.merge(&c).map_err(|_| (p, c)))
        .collect()
}

fn print_results(
    intervals: Vec<Interval<usize>>,
    lines: Vec<String>,
    line_number: bool,
) {
    for interval in intervals {
        for (line_no, line) in lines
            .iter()
            .enumerate()
            .take(interval.end + 1)
            .skip(interval.start)
        {
            if line_number {
                print!("{}: ", line_no + 1);
            }
            println!("{}", line);
        }
    }
}

fn read_file(file: impl Read) -> Vec<String> {
    BufReader::new(file).lines().map_while(Result::ok).collect()
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Prefix each line of output with the 1-based line number within its
    /// input file.
    #[arg(short, long, default_value_t = false)]
    line_number: bool,

    /// Print num lines of trailing context before matching lines.
    #[arg(short, long, default_value_t = 0, value_name = "num")]
    before_context: u8,

    /// Print num lines of trailing context after matching lines.
    #[arg(short, long, default_value_t = 0, value_name = "num")]
    after_context: u8,

    /// The regular expression to match.
    #[arg(required = true)]
    pattern: String,

    /// List of files to search.
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

// Result from a thread
struct RustleSuccess {
    intervals: Vec<Interval<usize>>,
    lines: Vec<String>,
}

// Result from a failed thread
struct RustleFailure {
    error: String,
}

fn main() {
    let cli = Cli::parse();

    // get values from clap
    let pattern = cli.pattern;
    let line_number = cli.line_number;
    let before_context = cli.before_context as usize;
    let after_context = cli.after_context as usize;
    let files = cli.files;

    // compile the regular expression
    let regex = match Regex::new(&pattern) {
        Ok(re) => re, // bind re to regex
        Err(e) => {
            eprintln!("{e}"); // write to standard error
            exit(1);
        }
    };

    thread::scope(|s| {
        let handles: Vec<_> = files
            .iter()
            .map(|file| {
                let filename = match file.to_str() {
                    Some(filename) => filename,
                    None => {
                        return Err(RustleFailure {
                            error: format!(
                                "Invalid filename: {}",
                                file.display()
                            ),
                        })
                    }
                };

                // attempt to open the file
                File::open(filename).map_err(|e| RustleFailure {
                    error: format!("Error opening {filename}: {e}"),
                })
            })
            .map_ok(|file| {
                // only spawn a thread for accessible file
                s.spawn(|| {
                    let lines = read_file(file);

                    // store the 0-based line number for any matched line
                    let match_lines = find_matching_lines(&lines, &regex);

                    // create intervals of the form [a,b] with the before/after context
                    let intervals = match create_intervals(
                        match_lines,
                        before_context,
                        after_context,
                    ) {
                        Ok(intervals) => intervals,
                        Err(_) => return Err(RustleFailure {
                            error: String::from(
                                "An error occurred while creating intervals",
                            ),
                        }),
                    };

                    // merge overlapping intervals
                    let intervals = merge_intervals(intervals);
                    Ok(RustleSuccess { intervals, lines })
                })
            })
            .collect();

        // process all the results
        for handle in handles {
            let result = match handle {
                Ok(scoped_join_handle) => scoped_join_handle,
                Err(e) => {
                    eprintln!("{}", e.error);
                    continue;
                }
            };

            if let Ok(result) = result.join() {
                match result {
                    Ok(result) => print_results(
                        result.intervals,
                        result.lines,
                        line_number,
                    ),
                    Err(e) => eprintln!("{}", e.error),
                };
            };
        }
    });
}
