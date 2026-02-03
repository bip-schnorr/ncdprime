use ncdprime_core::{Gzip, NcdOptions, ncd, ncd_matrix};

#[test]
fn identity_is_smallish() {
    let c = Gzip::new(6);
    let x = b"the quick brown fox jumps over the lazy dog\n".repeat(50);
    let d = ncd(&c, &x, &x, NcdOptions::default()).unwrap();
    // With real compressors and framing overhead, this is only "close" to 0.
    assert!(d >= 0.0);
    assert!(d < 0.6);
}

#[test]
fn matrix_matches_scalar() {
    let c = Gzip::new(6);
    // includes duplicates to exercise the singleton-size cache
    let a = vec![b"aaa".to_vec(), b"bbb".to_vec(), b"aaa".to_vec()];
    let b = vec![b"aaa".to_vec(), b"aaa".to_vec()];

    let m = ncd_matrix(&c, &a, &b, NcdOptions::default()).unwrap();
    assert_eq!(m.len(), 3);
    assert_eq!(m[0].len(), 2);

    // spot-check a few cells against scalar ncd()
    let d00 = ncd(&c, &a[0], &b[0], NcdOptions::default()).unwrap();
    let d10 = ncd(&c, &a[1], &b[0], NcdOptions::default()).unwrap();
    let d22 = ncd(&c, &a[2], &b[1], NcdOptions::default()).unwrap();

    assert!((m[0][0] - d00).abs() < 1e-12);
    assert!((m[1][0] - d10).abs() < 1e-12);
    assert!((m[2][1] - d22).abs() < 1e-12);
}
