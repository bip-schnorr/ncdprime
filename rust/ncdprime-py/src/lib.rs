use ncdprime_core::{Gzip, NcdOptions};
use pyo3::prelude::*;

/// Compute NCD between two byte strings.
///
/// Default behavior matches the "safe" mode we discussed:
/// - join=frame64
/// - symmetry=min(C(xy), C(yx))
#[pyfunction]
#[pyo3(signature = (x, y, gzip_level=None))]
fn ncd(x: &[u8], y: &[u8], gzip_level: Option<u32>) -> PyResult<f64> {
    let c = Gzip::new(gzip_level.unwrap_or(6));
    let d = ncdprime_core::ncd(&c, x, y, NcdOptions::default()).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("ncd failed: {e}"))
    })?;
    Ok(d)
}

#[pymodule]
fn ncdprime(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ncd, m)?)?;
    Ok(())
}
