use crate::args::{Args, Sub};
use clap::Parser;
use sig_pattern::pattern::Pattern;
use std::io::BufRead;
use std::str::FromStr;

mod args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    handle_args(args)?;
    Ok(())
}

fn handle_args(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    match args.sub {
        Sub::Count { pattern } => {
            let sig = Pattern::from_str(&pattern)?;
            println!("{}", sig.iter().len());
        },
        Sub::Format { pattern } => {
            println!("{}", Pattern::from_str(&pattern)?);
        },
        Sub::Merge { mut patterns, file } => {
            if let Some(f) = file {
                patterns.extend(get_file_lines(f)?);
            }
            println!("{}", merge_patterns(&patterns));
        },
    }
    Ok(())
}

fn get_file_lines(file: impl AsRef<std::path::Path>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(file)?;
    let reader = std::io::BufReader::new(file);
    Ok(reader.lines().map_while(Result::ok).collect())
}

fn merge_patterns(items: &[String]) -> Pattern {
    if items.is_empty() {
        return Pattern::default();
    }

    let patterns: Vec<Pattern> = items
        .iter()
        .filter_map(|s| {
            Pattern::from_str(s)
                .inspect_err(|e| eprintln!("Skipped invalid pattern '{s}': {e}"))
                .ok()
        })
        .collect();

    let mut result = patterns[0].clone();
    for p in patterns.iter().skip(1) {
        result = result.merge(p);
    }
    result
}
