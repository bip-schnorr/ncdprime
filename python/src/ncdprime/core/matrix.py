from __future__ import annotations

import hashlib
import json
import os
import pathlib
from dataclasses import asdict, dataclass
from typing import Iterable


@dataclass(frozen=True)
class MatrixSpec:
    rows: int
    cols: int
    min_bytes: int
    max_bytes: int
    seed: int = 0
    pattern: str = "gradient"  # gradient|row|col|constant


def _lerp(a: float, b: float, t: float) -> float:
    return a + (b - a) * t


def iter_cells(spec: MatrixSpec) -> Iterable[tuple[int, int, int]]:
    """Yield (r, c, nbytes) for each cell."""
    if spec.rows <= 0 or spec.cols <= 0:
        return

    for r in range(spec.rows):
        for c in range(spec.cols):
            if spec.pattern == "constant":
                t = 0.0
            elif spec.pattern == "row":
                t = 0.0 if spec.rows == 1 else r / (spec.rows - 1)
            elif spec.pattern == "col":
                t = 0.0 if spec.cols == 1 else c / (spec.cols - 1)
            else:
                # gradient
                denom = (spec.rows - 1) + (spec.cols - 1)
                t = 0.0 if denom <= 0 else (r + c) / denom

            n = int(round(_lerp(float(spec.min_bytes), float(spec.max_bytes), t)))
            n = max(0, n)
            yield (r, c, n)


def _cell_dirname(r: int, c: int) -> str:
    return f"r{r:03d}_c{c:03d}"


def _det_bytes(seed: int, r: int, c: int, n: int) -> bytes:
    """Deterministic pseudo-random bytes via hashing."""
    if n <= 0:
        return b""
    out = bytearray()
    counter = 0
    while len(out) < n:
        h = hashlib.blake2b(
            f"ncdprime:{seed}:{r}:{c}:{counter}".encode("utf-8"), digest_size=32
        ).digest()
        out.extend(h)
        counter += 1
    return bytes(out[:n])


def generate_matrix(
    *,
    outdir: pathlib.Path,
    spec: MatrixSpec,
    overwrite: bool = False,
    dry_run: bool = False,
) -> None:
    """Create a directory matrix dataset.

    Layout:
      outdir/
        matrix.json
        cells/
          r000_c000/payload.bin
          ...

    `matrix.json` captures the spec and cell sizes for reproducibility.
    """
    outdir = outdir.resolve()
    cells_dir = outdir / "cells"

    if outdir.exists():
        if not overwrite:
            raise typer.BadParameter(
                f"{outdir} exists. Pass --overwrite to replace it.", param_hint="outdir"
            )
        if not dry_run:
            # safer than rm -rf: delete known subpaths only.
            for root, dirs, files in os.walk(outdir, topdown=False):
                for fn in files:
                    pathlib.Path(root, fn).unlink(missing_ok=True)
                for dn in dirs:
                    pathlib.Path(root, dn).rmdir()
            outdir.rmdir()

    plan: list[dict[str, int]] = []
    for r, c, n in iter_cells(spec):
        plan.append({"row": r, "col": c, "bytes": n})

    if dry_run:
        print(json.dumps({"outdir": str(outdir), "spec": asdict(spec), "cells": plan[:5]}, indent=2))
        if len(plan) > 5:
            print(f"... plus {len(plan) - 5} more cells")
        return

    outdir.mkdir(parents=True, exist_ok=True)
    cells_dir.mkdir(parents=True, exist_ok=True)

    for cell in plan:
        r, c, n = cell["row"], cell["col"], cell["bytes"]
        d = cells_dir / _cell_dirname(r, c)
        d.mkdir(parents=True, exist_ok=True)
        (d / "payload.bin").write_bytes(_det_bytes(spec.seed, r, c, n))

    meta = {
        "format": "ncdprime.matrix.v1",
        "spec": asdict(spec),
        "cells": plan,
    }
    (outdir / "matrix.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")


# Local import to avoid making Typer a core dependency in other contexts.
import typer  # noqa: E402
