from __future__ import annotations

from dataclasses import dataclass

from ncdprime.compressors.base import Compressor


@dataclass(frozen=True)
class NcdResult:
    c_x: int
    c_y: int
    c_xy: int
    ncd: float


def compute_ncd(*, compressor: Compressor, x: bytes, y: bytes) -> NcdResult:
    """Compute NCD(x,y) using the given compressor.

    NCD(x,y) = (C(xy) - min(C(x), C(y))) / max(C(x), C(y))

    Notes:
      - This is a practical approximation; compressors are not ideal Kolmogorov estimators.
      - We use concatenation x + b"\0" + y as a delimiter.
    """
    rx = compressor.compress(x)
    ry = compressor.compress(y)
    rxy = compressor.compress(x + b"\0" + y)

    c_x, c_y, c_xy = rx.compressed_bytes, ry.compressed_bytes, rxy.compressed_bytes
    denom = max(c_x, c_y)
    ncd = 0.0 if denom <= 0 else (c_xy - min(c_x, c_y)) / denom
    return NcdResult(c_x=c_x, c_y=c_y, c_xy=c_xy, ncd=float(ncd))
