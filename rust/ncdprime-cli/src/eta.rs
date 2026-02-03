use std::time::Duration;

/// One timing sample for a completed cell.
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub input_bytes: u64,
    pub wall: Duration,
}

/// A tiny ETA estimator for long-running NCD grids.
///
/// Model: linear fit t ≈ a + b*n (least squares).
///
/// Why linear:
/// - robust and monotone in typical compression workloads
/// - easy to update and explain
///
/// Planned follow-up:
/// - optionally add power-law fit and pick by R² (like the Python estimator)
#[derive(Debug, Default, Clone)]
pub struct EtaEstimator {
    samples: Vec<Sample>,
    fit: Option<(f64, f64)>, // (a, b)
}

impl EtaEstimator {
    pub fn add(&mut self, s: Sample) {
        if s.input_bytes == 0 {
            return;
        }
        if s.wall.is_zero() {
            return;
        }
        self.samples.push(s);
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn should_refit(&self) -> bool {
        matches!(self.samples.len(), 6 | 15 | 16)
    }

    pub fn refit_first_n(&mut self, n: usize) {
        let n = n.max(2).min(self.samples.len());
        if n < 2 {
            self.fit = None;
            return;
        }
        let xs: Vec<f64> = self.samples.iter().take(n).map(|s| s.input_bytes as f64).collect();
        let ys: Vec<f64> = self
            .samples
            .iter()
            .take(n)
            .map(|s| s.wall.as_secs_f64())
            .collect();

        let xbar = xs.iter().sum::<f64>() / xs.len() as f64;
        let ybar = ys.iter().sum::<f64>() / ys.len() as f64;

        let sxx = xs.iter().map(|x| (x - xbar) * (x - xbar)).sum::<f64>();
        let b = if sxx <= 0.0 {
            0.0
        } else {
            let sxy = xs
                .iter()
                .zip(ys.iter())
                .map(|(x, y)| (x - xbar) * (y - ybar))
                .sum::<f64>();
            sxy / sxx
        };
        let a = ybar - b * xbar;
        self.fit = Some((a, b));
    }

    pub fn predict(&self, input_bytes: u64) -> Option<Duration> {
        let (a, b) = self.fit?;
        if input_bytes == 0 {
            return None;
        }
        let t = (a + b * (input_bytes as f64)).max(0.0);
        Some(Duration::from_secs_f64(t))
    }

    pub fn estimate_remaining<I: IntoIterator<Item = u64>>(&self, remaining: I) -> Option<Duration> {
        let mut total = 0.0;
        let mut any = false;
        let (a, b) = self.fit?;
        for n in remaining {
            if n == 0 {
                continue;
            }
            any = true;
            total += (a + b * (n as f64)).max(0.0);
        }
        if any {
            Some(Duration::from_secs_f64(total))
        } else {
            None
        }
    }
}
