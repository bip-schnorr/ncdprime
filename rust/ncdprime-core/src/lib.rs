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

pub fn ncd<C: Compressor>(c: &C, x: &[u8], y: &[u8], opts: NcdOptions) -> io::Result<f64> {
    let cx = c.compressed_len(x)? as f64;
    let cy = c.compressed_len(y)? as f64;
    ncd_from_sizes(c, x, y, cx, cy, opts)
}

fn ncd_from_sizes<C: Compressor>(
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
        if d < 0.0 {
            d = 0.0;
        }
        if d > 1.0 {
            d = 1.0;
        }
    }

    Ok(d)
}

/// Compute an NCD matrix between two sets of byte buffers.
///
/// This function caches C(x) / C(y) so computing a matrix isn't O((n*m) * compress(x)).
pub fn ncd_matrix<C: Compressor>(
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

pub fn read_all<R: Read>(mut r: R) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)?;
    Ok(buf)
}
