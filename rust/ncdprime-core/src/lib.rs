mod factory;

pub use factory::{CompressorSpec, parse_compressor};

use std::collections::HashMap;
use std::io::{self, Read, Write};

#[derive(Clone, Copy, Debug)]
pub enum Join {
    /// frame64(x) || frame64(y), where frame64(b) = u64_le(len) || b
    Frame64,
}

fn frame64_bytes(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + data.len());
    let len: u64 = data.len().try_into().unwrap_or(u64::MAX);
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(data);
    out
}

pub fn join_bytes(x: &[u8], y: &[u8], join: Join) -> Vec<u8> {
    match join {
        Join::Frame64 => {
            let fx = frame64_bytes(x);
            let fy = frame64_bytes(y);
            let mut out = Vec::with_capacity(fx.len() + fy.len());
            out.extend_from_slice(&fx);
            out.extend_from_slice(&fy);
            out
        }
    }
}

pub trait Compressor {
    fn id(&self) -> &'static str;
    fn compressed_len(&self, input: &[u8]) -> io::Result<usize>;
}

/// A deterministic gzip compressor.
///
/// We implement gzip via flate2's GzEncoder and set the header fields deterministically.
pub struct Gzip {
    level: flate2::Compression,
}

impl Gzip {
    pub fn new(level: u32) -> Self {
        Self {
            level: flate2::Compression::new(level),
        }
    }
}

/// Zstandard compressor.
pub struct Zstd {
    level: i32,
}

impl Zstd {
    pub fn new(level: i32) -> Self {
        Self { level }
    }
}

impl Compressor for Zstd {
    fn id(&self) -> &'static str {
        "zstd"
    }

    fn compressed_len(&self, input: &[u8]) -> io::Result<usize> {
        zstd::stream::encode_all(input, self.level)
            .map(|v| v.len())
            .map_err(io::Error::other)
    }
}

/// Brotli compressor.
pub struct Brotli {
    quality: u32,
    /// window size as a power of two (10..=24)
    lgwin: u32,
}

impl Brotli {
    pub fn new(quality: u32, lgwin: u32) -> Self {
        Self { quality, lgwin }
    }
}

impl Compressor for Brotli {
    fn id(&self) -> &'static str {
        "brotli"
    }

    fn compressed_len(&self, input: &[u8]) -> io::Result<usize> {
        use brotli::enc::BrotliEncoderParams;

        let mut out = Vec::new();
        let params = BrotliEncoderParams {
            quality: self.quality as i32,
            lgwin: self.lgwin as i32,
            ..Default::default()
        };

        brotli::BrotliCompress(&mut &input[..], &mut out, &params)
            .map_err(|e| io::Error::other(format!("brotli: {e:?}")))?;

        Ok(out.len())
    }
}

/// LZ4 (block) compressor.
pub struct Lz4 {
    /// acceleration factor: 1 = best compression, higher = faster
    accel: i32,
}

impl Lz4 {
    pub fn new(accel: i32) -> Self {
        Self { accel }
    }
}

impl Compressor for Lz4 {
    fn id(&self) -> &'static str {
        "lz4"
    }

    fn compressed_len(&self, input: &[u8]) -> io::Result<usize> {
        // lz4_flex doesn't expose acceleration; it uses a fast algorithm.
        // We'll treat this as a placeholder for "lz4" and keep determinism.
        let _ = self.accel;
        Ok(lz4_flex::compress_prepend_size(input).len())
    }
}

/// XZ (LZMA2) compressor.
pub struct Xz {
    level: u32,
}

impl Xz {
    pub fn new(level: u32) -> Self {
        Self { level }
    }
}

impl Compressor for Xz {
    fn id(&self) -> &'static str {
        "xz"
    }

    fn compressed_len(&self, input: &[u8]) -> io::Result<usize> {
        use xz2::write::XzEncoder;

        let mut enc = XzEncoder::new(Vec::new(), self.level);
        enc.write_all(input)?;
        let out = enc.finish()?;
        Ok(out.len())
    }
}

impl Compressor for Gzip {
    fn id(&self) -> &'static str {
        "gzip"
    }

    fn compressed_len(&self, input: &[u8]) -> io::Result<usize> {
        use flate2::GzBuilder;

        let mut enc = GzBuilder::new().mtime(0).write(Vec::new(), self.level);
        enc.write_all(input)?;
        let out = enc.finish()?;
        Ok(out.len())
    }
}

pub fn compressor_ids() -> &'static [&'static str] {
    &["gzip", "zstd", "brotli", "lz4", "xz"]
}

#[derive(Clone, Copy, Debug)]
pub enum Symmetry {
    None,
    Min,
}

#[derive(Clone, Copy, Debug)]
pub struct NcdOptions {
    pub join: Join,
    pub symmetry: Symmetry,
    pub clamp_0_1: bool,
}

impl Default for NcdOptions {
    fn default() -> Self {
        Self {
            join: Join::Frame64,
            symmetry: Symmetry::Min,
            clamp_0_1: false,
        }
    }
}

pub fn ncd<C: Compressor + ?Sized>(c: &C, x: &[u8], y: &[u8], opts: NcdOptions) -> io::Result<f64> {
    let cx = c.compressed_len(x)? as f64;
    let cy = c.compressed_len(y)? as f64;
    ncd_from_sizes(c, x, y, cx, cy, opts)
}

fn ncd_from_sizes<C: Compressor + ?Sized>(
    c: &C,
    x: &[u8],
    y: &[u8],
    cx: f64,
    cy: f64,
    opts: NcdOptions,
) -> io::Result<f64> {
    let min = cx.min(cy);
    let max = cx.max(cy);

    if max == 0.0 {
        return Ok(0.0);
    }

    let cxy = c.compressed_len(&join_bytes(x, y, opts.join))? as f64;
    let ccat = match opts.symmetry {
        Symmetry::None => cxy,
        Symmetry::Min => {
            let cyx = c.compressed_len(&join_bytes(y, x, opts.join))? as f64;
            cxy.min(cyx)
        }
    };

    let mut d = (ccat - min) / max;
    if d.is_nan() {
        d = 0.0;
    }

    if opts.clamp_0_1 {
        d = d.clamp(0.0, 1.0);
    }

    Ok(d)
}

/// Compute an NCD matrix between two sets of byte buffers.
///
/// This function caches C(x) / C(y) so computing a matrix isn't O((n*m) * compress(x)).
pub fn ncd_matrix<C: Compressor + ?Sized>(
    c: &C,
    a: &[Vec<u8>],
    b: &[Vec<u8>],
    opts: NcdOptions,
) -> io::Result<Vec<Vec<f64>>> {
    // Deduplicate singleton compression sizes using a content hash.
    // This helps when the same bytes appear multiple times in `a` and/or `b`.
    let mut size_cache: HashMap<[u8; 32], f64> = HashMap::new();

    let mut a_sizes: Vec<f64> = Vec::with_capacity(a.len());
    for x in a {
        let key: [u8; 32] = *blake3::hash(x).as_bytes();
        let cx = match size_cache.get(&key) {
            Some(v) => *v,
            None => {
                let v = c.compressed_len(x)? as f64;
                size_cache.insert(key, v);
                v
            }
        };
        a_sizes.push(cx);
    }

    let mut b_sizes: Vec<f64> = Vec::with_capacity(b.len());
    for y in b {
        let key: [u8; 32] = *blake3::hash(y).as_bytes();
        let cy = match size_cache.get(&key) {
            Some(v) => *v,
            None => {
                let v = c.compressed_len(y)? as f64;
                size_cache.insert(key, v);
                v
            }
        };
        b_sizes.push(cy);
    }

    let mut out = vec![vec![0.0; b.len()]; a.len()];

    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            out[i][j] = ncd_from_sizes(c, x, y, a_sizes[i], b_sizes[j], opts)?;
        }
    }

    Ok(out)
}

/// Per-cell progress information for `ncd_matrix_with_progress`.
#[derive(Clone, Copy, Debug)]
pub struct NcdMatrixProgress {
    pub done: usize,
    pub total: usize,
    /// Rough "work" proxy for estimation; currently `x.len() + y.len()`.
    pub input_bytes: u64,
    pub wall: std::time::Duration,
}

/// Compute an NCD matrix, invoking a callback after each computed cell.
///
/// Intended for CLIs to display progress + ETA.
pub fn ncd_matrix_with_progress<C: Compressor + ?Sized, F>(
    c: &C,
    a: &[Vec<u8>],
    b: &[Vec<u8>],
    opts: NcdOptions,
    mut on_cell: F,
) -> io::Result<Vec<Vec<f64>>>
where
    F: FnMut(NcdMatrixProgress),
{
    // Keep the same caching behavior as `ncd_matrix`.
    let mut size_cache: HashMap<[u8; 32], f64> = HashMap::new();

    let mut a_sizes: Vec<f64> = Vec::with_capacity(a.len());
    for x in a {
        let key: [u8; 32] = *blake3::hash(x).as_bytes();
        let cx = match size_cache.get(&key) {
            Some(v) => *v,
            None => {
                let v = c.compressed_len(x)? as f64;
                size_cache.insert(key, v);
                v
            }
        };
        a_sizes.push(cx);
    }

    let mut b_sizes: Vec<f64> = Vec::with_capacity(b.len());
    for y in b {
        let key: [u8; 32] = *blake3::hash(y).as_bytes();
        let cy = match size_cache.get(&key) {
            Some(v) => *v,
            None => {
                let v = c.compressed_len(y)? as f64;
                size_cache.insert(key, v);
                v
            }
        };
        b_sizes.push(cy);
    }

    let total = a.len().saturating_mul(b.len());
    let mut done = 0usize;

    let mut out = vec![vec![0.0; b.len()]; a.len()];

    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            let start = std::time::Instant::now();
            out[i][j] = ncd_from_sizes(c, x, y, a_sizes[i], b_sizes[j], opts)?;
            let wall = start.elapsed();

            done = done.saturating_add(1);
            on_cell(NcdMatrixProgress {
                done,
                total,
                input_bytes: (x.len() + y.len()) as u64,
                wall,
            });
        }
    }

    Ok(out)
}

pub fn read_all<R: Read>(mut r: R) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)?;
    Ok(buf)
}
