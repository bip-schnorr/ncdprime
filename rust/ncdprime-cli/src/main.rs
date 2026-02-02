use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "ncdprime")]
#[command(about = "Modern NCD (Normalized Compression Distance) utility", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compute NCD between two files
    Pair {
        file_a: String,
        file_b: String,
        #[arg(long, default_value_t = 6)]
        gzip_level: u32,
        /// gzip header mtime (timestamp). Use 0 for deterministic output.
        #[arg(long, default_value_t = 0)]
        gzip_mtime: u32,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pair {
            file_a,
            file_b,
            gzip_level,
            gzip_mtime,
        } => {
            let a = fs::read(file_a)?;
            let b = fs::read(file_b)?;
            let c = ncdprime_core::Gzip::with_mtime(gzip_level, gzip_mtime);
            let d = ncdprime_core::ncd(&c, &a, &b, ncdprime_core::NcdOptions::default())?;
            println!("{d}");
        }
    }

    Ok(())
}
