from __future__ import annotations

from importlib.metadata import entry_points
from typing import Iterable, cast

from ncdprime.compressors.base import Compressor
from ncdprime.compressors.gzipc import GzipCompressor


def iter_compressor_factories() -> Iterable[tuple[str, object]]:
    """Yield (name, factory) for built-ins + entry points.

    Entry points under group `ncdprime.compressors` may point to a class or factory callable.
    """
    yield ("gzip", GzipCompressor)

    for ep in entry_points().select(group="ncdprime.compressors"):
        try:
            obj = ep.load()
        except Exception:
            continue
        yield (ep.name, obj)


def get_compressor(name: str) -> Compressor:
    name = name.strip()
    for n, factory in iter_compressor_factories():
        if n == name:
            obj = factory() if callable(factory) else factory
            return cast(Compressor, obj)
    raise KeyError(f"Unknown compressor: {name}")
