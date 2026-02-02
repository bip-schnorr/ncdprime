use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn ncd(x: Buffer, y: Buffer, gzip_level: Option<u32>, gzip_mtime: Option<u32>) -> Result<f64> {
    let c = ncdprime_core::Gzip::with_mtime(gzip_level.unwrap_or(6), gzip_mtime.unwrap_or(0));
    let d = ncdprime_core::ncd(&c, &x, &y, ncdprime_core::NcdOptions::default())
        .map_err(|e| Error::from_reason(format!("ncd failed: {e}")))?;
    Ok(d)
}

#[napi]
pub fn matrix(
    a: Vec<Buffer>,
    b: Vec<Buffer>,
    gzip_level: Option<u32>,
    gzip_mtime: Option<u32>,
) -> Result<Vec<Vec<f64>>> {
    let c = ncdprime_core::Gzip::with_mtime(gzip_level.unwrap_or(6), gzip_mtime.unwrap_or(0));

    let a_vecs: Vec<Vec<u8>> = a.into_iter().map(|buf| buf.to_vec()).collect();
    let b_vecs: Vec<Vec<u8>> = b.into_iter().map(|buf| buf.to_vec()).collect();

    let out = ncdprime_core::ncd_matrix(&c, &a_vecs, &b_vecs, ncdprime_core::NcdOptions::default())
        .map_err(|e| Error::from_reason(format!("matrix failed: {e}")))?;

    Ok(out)
}
