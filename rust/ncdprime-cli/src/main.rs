mod inputs;
mod matrix;

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
    },

    /// Compute an NCD matrix between two sets (dirs, files, list files, or literals).
    Matrix {
        set_a: String,
        set_b: Option<String>,
        #[arg(long, default_value_t = false)]
        square: bool,
        /// Interpret set args as newline-separated file-list files
        #[arg(long, default_value_t = false)]
        list: bool,
        /// Output format (tsv|csv)
        #[arg(long, default_value = "tsv")]
        format: String,
        /// Omit row/column labels
        #[arg(long = "no-labels", default_value_t = false)]
        no_labels: bool,
        #[arg(long, default_value_t = 6)]
        gzip_level: u32,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pair {
            file_a,
            file_b,
            gzip_level,
        } => {
            let a = fs::read(file_a)?;
            let b = fs::read(file_b)?;
            let c = ncdprime_core::Gzip::new(gzip_level);
            let d = ncdprime_core::ncd(&c, &a, &b, ncdprime_core::NcdOptions::default())?;
            println!("{d}");
        }

        Commands::Matrix {
            set_a,
            set_b,
            square,
            list,
            format,
            no_labels,
            gzip_level,
        } => {
            let spec_a = inputs::auto_detect_set_spec(&set_a, list)?;
            let spec_b = if square {
                spec_a.clone()
            } else {
                inputs::auto_detect_set_spec(set_b.as_deref().unwrap_or(&set_a), list)?
            };

            let a = inputs::load_set(&spec_a)?;
            let b = inputs::load_set(&spec_b)?;

            let c = ncdprime_core::Gzip::new(gzip_level);

            let a_bytes: Vec<Vec<u8>> = a.items.iter().map(|i| i.bytes.clone()).collect();
            let b_bytes: Vec<Vec<u8>> = b.items.iter().map(|i| i.bytes.clone()).collect();

            let values = ncdprime_core::ncd_matrix(
                &c,
                &a_bytes,
                &b_bytes,
                ncdprime_core::NcdOptions::default(),
            )?;
            let (rows, cols) = matrix::rows_cols(&a, &b);
            let out = matrix::format_matrix(
                &rows,
                &cols,
                &values,
                if format == "csv" { "csv" } else { "tsv" },
                !no_labels,
            );
            print!("{out}");
        }
    }

    Ok(())
}
