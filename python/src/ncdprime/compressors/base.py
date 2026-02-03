from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol


@dataclass(frozen=True)
class CompressResult:
    compressed_bytes: int
    wall_time_s: float


class Compressor(Protocol):
    """Compressor plugin interface."""

    name: str

    def compress(self, data: bytes) -> CompressResult: ...
