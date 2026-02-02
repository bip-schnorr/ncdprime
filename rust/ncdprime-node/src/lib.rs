use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn ncd(x: Buffer, y: Buffer, gzip_level: Option<u32>) -> Result<f64> {
    let c = ncdprime_core::Gzip::new(gzip_level.unwrap_or(6));
    let d = ncdprime_core::ncd(&c, &x, &y, ncdprime_core::NcdOptions::default())
        .map_err(|e| Error::from_reason(format!("ncd failed: {e}")))?;
    Ok(d)
}
