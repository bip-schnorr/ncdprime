use std::time::Duration;

use ncdprime_cli::eta::{EtaEstimator, Sample};

#[test]
fn eta_should_refit_at_6_and_15ish() {
    let mut est = EtaEstimator::default();

    for i in 0..5 {
        est.add(Sample {
            input_bytes: 10_000 + i,
            wall: Duration::from_millis(10),
        });
        assert!(!est.should_refit());
    }

    est.add(Sample {
        input_bytes: 20_000,
        wall: Duration::from_millis(10),
    });
    assert!(est.should_refit());

    // Fill to 15
    while est.sample_count() < 15 {
        est.add(Sample {
            input_bytes: 30_000,
            wall: Duration::from_millis(10),
        });
    }
    assert!(est.should_refit());
}

#[test]
fn eta_estimate_remaining_is_monotone_in_bytes() {
    let mut est = EtaEstimator::default();

    // Linear-ish samples
    for n in [10_000u64, 50_000, 100_000, 200_000, 400_000, 800_000] {
        est.add(Sample {
            input_bytes: n,
            wall: Duration::from_secs_f64(0.05 + 2e-6 * n as f64),
        });
    }
    est.refit_first_n(6);

    let small = est.estimate_remaining([10_000u64, 10_000]).unwrap();
    let big = est.estimate_remaining([100_000u64, 100_000]).unwrap();
    assert!(big > small);
}
