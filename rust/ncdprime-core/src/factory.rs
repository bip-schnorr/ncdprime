use crate::{Brotli, Compressor, Gzip, Lz4, Xz, Zstd};
use std::io;

#[derive(Clone, Debug)]
pub enum CompressorSpec {
    Gzip { level: u32 },
    Zstd { level: i32 },
    Brotli { quality: u32, lgwin: u32 },
    Lz4 { accel: i32 },
    Xz { level: u32 },
}

impl Default for CompressorSpec {
    fn default() -> Self {
        CompressorSpec::Gzip { level: 9 }
    }
}

impl CompressorSpec {
    pub fn id(&self) -> &'static str {
        match self {
            CompressorSpec::Gzip { .. } => "gzip",
            CompressorSpec::Zstd { .. } => "zstd",
            CompressorSpec::Brotli { .. } => "brotli",
            CompressorSpec::Lz4 { .. } => "lz4",
            CompressorSpec::Xz { .. } => "xz",
        }
    }

    pub fn build(&self) -> Box<dyn Compressor> {
        match *self {
            CompressorSpec::Gzip { level } => Box::new(Gzip::new(level)),
            CompressorSpec::Zstd { level } => Box::new(Zstd::new(level)),
            CompressorSpec::Brotli { quality, lgwin } => Box::new(Brotli::new(quality, lgwin)),
            CompressorSpec::Lz4 { accel } => Box::new(Lz4::new(accel)),
            CompressorSpec::Xz { level } => Box::new(Xz::new(level)),
        }
    }
}

pub fn parse_compressor(
    id: &str,
    gzip_level: u32,
    zstd_level: i32,
    brotli_quality: u32,
    brotli_lgwin: u32,
    lz4_accel: i32,
    xz_level: u32,
) -> io::Result<CompressorSpec> {
    Ok(match id {
        "gzip" => CompressorSpec::Gzip { level: gzip_level },
        "zstd" => CompressorSpec::Zstd { level: zstd_level },
        "brotli" => CompressorSpec::Brotli {
            quality: brotli_quality,
            lgwin: brotli_lgwin,
        },
        "lz4" => CompressorSpec::Lz4 { accel: lz4_accel },
        "xz" => CompressorSpec::Xz { level: xz_level },
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown compressor id: {id}"),
            ))
        }
    })
}
