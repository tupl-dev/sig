use clap::{Parser, Subcommand};

#[derive(Clone, Debug, Parser)]
#[command(name = "sig", about = "Byte signature tool")]
pub struct Args {
    #[command(subcommand)]
    pub sub: Sub,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Sub {
    #[command(name = "count", about = "Count the number of bytes in a pattern")]
    Count { pattern: String },

    #[command(name = "format", about = "Format a pattern")]
    Format { pattern: String },

    #[command(name = "merge", about = "Merge patterns")]
    Merge {
        patterns: Vec<String>,

        #[arg(short, long)]
        file: Option<std::path::PathBuf>,
    },
}
