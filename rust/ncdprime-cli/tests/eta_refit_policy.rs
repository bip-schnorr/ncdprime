use std::time::Duration;

use ncdprime_cli::eta::{EtaEstimator, Sample};

#[test]
fn refit_trigger_points_are_stable() {
    let mut est = EtaEstimator::default();

    for i in 0..16 {
        est.add(Sample {
            input_bytes: 10_000 + i,
            wall: Duration::from_millis(5),
        });
        let k = est.sample_count();
        let should = est.should_refit();
        if k == 6 || k == 15 || k == 16 {
            assert!(should);
        } else {
            assert!(!should);
        }
    }
}
