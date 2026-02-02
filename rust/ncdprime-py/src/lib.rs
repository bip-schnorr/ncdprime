use ncdprime_core::{Gzip, NcdOptions};
use pyo3::prelude::*;

/// Compute NCD between two byte strings.
///
/// Default behavior:
/// - join=frame64
/// - symmetry=min(C(xy), C(yx))
#[pyfunction]
#[pyo3(signature = (x, y, gzip_level=None, gzip_mtime=None))]
fn ncd(x: &[u8], y: &[u8], gzip_level: Option<u32>, gzip_mtime: Option<u32>) -> PyResult<f64> {
    let c = Gzip::with_mtime(gzip_level.unwrap_or(6), gzip_mtime.unwrap_or(0));
    let d = ncdprime_core::ncd(&c, x, y, NcdOptions::default()).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("ncd failed: {e}"))
    })?;
    Ok(d)
}

/// Compute an NCD matrix between two lists of byte strings.
///
/// Returns a list-of-lists of floats (rows = a, cols = b).
#[pyfunction]
#[pyo3(signature = (a, b, gzip_level=None, gzip_mtime=None))]
fn matrix(
    a: Vec<Vec<u8>>,
    b: Vec<Vec<u8>>,
    gzip_level: Option<u32>,
    gzip_mtime: Option<u32>,
) -> PyResult<Vec<Vec<f64>>> {
    let c = Gzip::with_mtime(gzip_level.unwrap_or(6), gzip_mtime.unwrap_or(0));
    let m = ncdprime_core::ncd_matrix(&c, &a, &b, NcdOptions::default()).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("matrix failed: {e}"))
    })?;
    Ok(m)
}

#[pymodule]
fn ncdprime(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ncd, m)?)?;
    m.add_function(wrap_pyfunction!(matrix, m)?)?;
    Ok(())
}
