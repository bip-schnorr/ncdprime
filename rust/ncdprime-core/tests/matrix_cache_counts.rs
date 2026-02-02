use std::sync::atomic::{AtomicUsize, Ordering};

use ncdprime_core::{ncd_matrix, Compressor, NcdOptions, Symmetry};

#[derive(Default)]
struct CountingCompressor {
    calls: AtomicUsize,
}

impl CountingCompressor {
    fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}

impl Compressor for CountingCompressor {
    fn id(&self) -> &'static str {
        "count"
    }

    fn compressed_len(&self, input: &[u8]) -> std::io::Result<usize> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        // A deterministic, cheap "compressor": size is just the input length.
        Ok(input.len())
    }
}

#[test]
fn matrix_singleton_cache_dedups_across_a_and_b_sym_min() {
    let c = CountingCompressor::default();

    // Unique blobs across (a ∪ b) are: {"aaa", "bbb"} => 2 singleton compressions expected.
    let a = vec![b"aaa".to_vec(), b"bbb".to_vec(), b"aaa".to_vec()];
    let b = vec![b"aaa".to_vec(), b"aaa".to_vec()];

    let _m = ncd_matrix(&c, &a, &b, NcdOptions::default()).unwrap();

    let n = a.len();
    let m = b.len();
    let unique_singletons = 2;
    let per_cell = 2; // symmetry=min => C(xy) and C(yx)
    let expected_calls = unique_singletons + per_cell * n * m;

    assert_eq!(c.calls(), expected_calls);
}

#[test]
fn matrix_singleton_cache_dedups_across_a_and_b_sym_none() {
    let c = CountingCompressor::default();

    // Unique blobs across (a ∪ b) are: {"aaa", "bbb"} => 2 singleton compressions expected.
    let a = vec![b"aaa".to_vec(), b"bbb".to_vec(), b"aaa".to_vec()];
    let b = vec![b"aaa".to_vec(), b"aaa".to_vec()];

    let mut opts = NcdOptions::default();
    opts.symmetry = Symmetry::None;

    let _m = ncd_matrix(&c, &a, &b, opts).unwrap();

    let n = a.len();
    let m = b.len();
    let unique_singletons = 2;
    let per_cell = 1; // symmetry=none => only C(xy)
    let expected_calls = unique_singletons + per_cell * n * m;

    assert_eq!(c.calls(), expected_calls);
}
