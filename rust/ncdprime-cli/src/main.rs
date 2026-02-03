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
        #[arg(long, default_value = "gzip")]
        compressor: String,
        #[arg(long, default_value_t = 9)]
        gzip_level: u32,
        #[arg(long, default_value_t = 3)]
        zstd_level: i32,
        #[arg(long, default_value_t = 11)]
        brotli_quality: u32,
        #[arg(long, default_value_t = 22)]
        brotli_lgwin: u32,
        #[arg(long, default_value_t = 1)]
        lz4_accel: i32,
        #[arg(long, default_value_t = 6)]
        xz_level: u32,
    },

    /// List available compressors
    Compressors,

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
        #[arg(long, default_value = "gzip")]
        compressor: String,
        #[arg(long, default_value_t = 9)]
        gzip_level: u32,
        #[arg(long, default_value_t = 3)]
        zstd_level: i32,
        #[arg(long, default_value_t = 11)]
        brotli_quality: u32,
        #[arg(long, default_value_t = 22)]
        brotli_lgwin: u32,
        #[arg(long, default_value_t = 1)]
        lz4_accel: i32,
        #[arg(long, default_value_t = 6)]
        xz_level: u32,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compressors => {
            for id in ncdprime_core::compressor_ids() {
                println!("{id}");
            }
        }

        Commands::Pair {
            file_a,
            file_b,
            compressor,
            gzip_level,
            zstd_level,
            brotli_quality,
            brotli_lgwin,
            lz4_accel,
            xz_level,
        } => {
            let a = fs::read(file_a)?;
            let b = fs::read(file_b)?;
            let spec = ncdprime_core::parse_compressor(
                &compressor,
                gzip_level,
                zstd_level,
                brotli_quality,
                brotli_lgwin,
                lz4_accel,
                xz_level,
            )?;
            let c = spec.build();
            let d = ncdprime_core::ncd(&*c, &a, &b, ncdprime_core::NcdOptions::default())?;
            println!("{d}");
        }

        Commands::Matrix {
            set_a,
            set_b,
            square,
            list,
            format,
            no_labels,
            compressor,
            gzip_level,
            zstd_level,
            brotli_quality,
            brotli_lgwin,
            lz4_accel,
            xz_level,
        } => {
            let spec_a = inputs::auto_detect_set_spec(&set_a, list)?;
            let spec_b = if square {
                spec_a.clone()
            } else {
                inputs::auto_detect_set_spec(set_b.as_deref().unwrap_or(&set_a), list)?
            };

            let a = inputs::load_set(&spec_a)?;
            let b = inputs::load_set(&spec_b)?;

            let spec = ncdprime_core::parse_compressor(
                &compressor,
                gzip_level,
                zstd_level,
                brotli_quality,
                brotli_lgwin,
                lz4_accel,
                xz_level,
            )?;
            let c = spec.build();

            let a_bytes: Vec<Vec<u8>> = a.items.iter().map(|i| i.bytes.clone()).collect();
            let b_bytes: Vec<Vec<u8>> = b.items.iter().map(|i| i.bytes.clone()).collect();

            let mut est = ncdprime_cli::eta::EtaEstimator::default();
            let started = std::time::Instant::now();
            let mut bytes_seen: u128 = 0;

            let values = ncdprime_core::ncd_matrix_with_progress(
                &*c,
                &a_bytes,
                &b_bytes,
                ncdprime_core::NcdOptions::default(),
                |p| {
                    // Keep the estimator warm.
                    est.add(ncdprime_cli::eta::Sample {
                        input_bytes: p.input_bytes,
                        wall: p.wall,
                    });
                    bytes_seen += p.input_bytes as u128;

                    if est.should_refit() {
                        est.refit_first_n(est.sample_count());
                    }

                    // Emit occasional progress updates to stderr (so matrix output stays clean).
                    let should_print = p.done == 1
                        || p.done == p.total
                        || (p.done % 200 == 0)
                        || est.should_refit();
                    if !should_print {
                        return;
                    }

                    let elapsed = started.elapsed();
                    let pct = if p.total == 0 {
                        100.0
                    } else {
                        (p.done as f64) * 100.0 / (p.total as f64)
                    };

                    let remaining_cells = p.total.saturating_sub(p.done);
                    let avg_bytes = if p.done == 0 {
                        0u64
                    } else {
                        (bytes_seen / (p.done as u128)) as u64
                    };
                    let eta =
                        est.estimate_remaining(std::iter::repeat_n(avg_bytes, remaining_cells));

                    fn fmt_dur(d: std::time::Duration) -> String {
                        let s = d.as_secs();
                        let h = s / 3600;
                        let m = (s % 3600) / 60;
                        let ss = s % 60;
                        if h > 0 {
                            format!("{h}:{m:02}:{ss:02}")
                        } else {
                            format!("{m}:{ss:02}")
                        }
                    }

                    let eta_str = eta.map(fmt_dur).unwrap_or_else(|| "?".to_string());
                    eprintln!(
                        "matrix: {}/{} ({pct:.1}%) elapsed={} eta={} (samples={})",
                        p.done,
                        p.total,
                        fmt_dur(elapsed),
                        eta_str,
                        est.sample_count(),
                    );
                },
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
