from __future__ import annotations

import gzip
import time

from ncdprime.compressors.base import CompressResult


class GzipCompressor:
    name = "gzip"

    def __init__(self, level: int = 6) -> None:
        self.level = level

    def compress(self, data: bytes) -> CompressResult:
        t0 = time.perf_counter()
        out = gzip.compress(data, compresslevel=self.level)
        t1 = time.perf_counter()
        return CompressResult(compressed_bytes=len(out), wall_time_s=t1 - t0)
