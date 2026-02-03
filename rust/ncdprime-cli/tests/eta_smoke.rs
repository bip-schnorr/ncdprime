use std::time::Duration;

use ncdprime_cli::eta::{EtaEstimator, Sample};

#[test]
fn eta_basic_linear_fit() {
    let mut est = EtaEstimator::default();
    for n in [10_000u64, 50_000, 100_000, 200_000, 400_000, 800_000] {
        est.add(Sample {
            input_bytes: n,
            wall: Duration::from_secs_f64(0.05 + 2e-6 * n as f64),
        });
    }
    est.refit_first_n(6);

    let pred = est.predict(300_000).unwrap().as_secs_f64();
    let truth = 0.05 + 2e-6 * 300_000f64;
    assert!((pred - truth).abs() < 0.05);

    let rem = est.estimate_remaining([1_000u64, 2_000, 3_000]).unwrap();
    assert!(rem.as_secs_f64() > 0.0);
}
