use ncdprime_core::{ncd, Gzip, NcdOptions};

#[test]
fn identity_is_smallish() {
    let c = Gzip::new(6);
    let x = b"the quick brown fox jumps over the lazy dog\n".repeat(50);
    let d = ncd(&c, &x, &x, NcdOptions::default()).unwrap();
    // With real compressors and framing overhead, this is only "close" to 0.
    assert!(d >= 0.0);
    assert!(d < 0.6);
}
