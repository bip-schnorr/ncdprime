from __future__ import annotations

import json
import pathlib
import time
from dataclasses import dataclass
from typing import Optional

import typer

from ncdprime.core.ncd import compute_ncd
from ncdprime.estimator import CompletionTimeEstimator, TimingSample
from ncdprime.io.dataset import DatasetItem, iter_matrix_items, load_bytes
from ncdprime.plugins.registry import get_compressor

app = typer.Typer(add_completion=False, no_args_is_help=True)


@dataclass(frozen=True)
class PairJob:
    a: DatasetItem
    b: DatasetItem
    input_bytes: int


def _format_seconds(s: float) -> str:
    s = max(0.0, float(s))
    if s < 60:
        return f"{s:.1f}s"
    m, sec = divmod(s, 60)
    if m < 60:
        return f"{int(m)}m{sec:04.1f}s"
    h, m = divmod(m, 60)
    return f"{int(h)}h{int(m):02d}m"


@app.command("matrix")
def run_matrix(
    matrix_dir: pathlib.Path = typer.Argument(..., help="Matrix dataset directory."),
    compressor: str = typer.Option("gzip", "--compressor", "-z", help="Compressor name."),
    out: pathlib.Path = typer.Option(
        pathlib.Path("results.jsonl"), "--out", help="Output JSONL file."
    ),
    max_item_bytes: Optional[int] = typer.Option(
        None, "--max-item-bytes", help="Truncate each item payload to this many bytes."
    ),
    pairs: str = typer.Option(
        "upper", "--pairs", help="Pairs to compute: upper|all (upper excludes i>j)."
    ),
    show_eta: bool = typer.Option(True, "--eta/--no-eta", help="Print ETA estimates."),
    progress: bool = typer.Option(True, "--progress/--no-progress", help="Show progress bar."),
) -> None:
    """Run NCD over a generated matrix dataset."""
    comp = get_compressor(compressor)
    items = list(iter_matrix_items(matrix_dir))
    if not items:
        raise typer.BadParameter(f"No items found under {matrix_dir}")

    if pairs not in {"upper", "all"}:
        raise typer.BadParameter("pairs must be upper or all")

    out = out.resolve()
    out.parent.mkdir(parents=True, exist_ok=True)

    # Preload items (small datasets are expected for now).
    payloads = {it.key: load_bytes(it.path, max_bytes=max_item_bytes) for it in items}

    jobs: list[PairJob] = []
    n = len(items)
    for i in range(n):
        for j in range(n):
            if pairs == "upper" and j < i:
                continue
            a, b = items[i], items[j]
            jobs.append(PairJob(a=a, b=b, input_bytes=len(payloads[a.key]) + len(payloads[b.key])))

    est = CompletionTimeEstimator()

    # Rich progress is optional; fall back to periodic prints.
    prog = None
    task_id = None
    if progress:
        from rich.progress import (
            BarColumn,
            Progress,
            SpinnerColumn,
            TextColumn,
            TimeElapsedColumn,
        )

        prog = Progress(
            SpinnerColumn(),
            TextColumn("{task.description}"),
            BarColumn(),
            TextColumn("{task.completed}/{task.total}"),
            TimeElapsedColumn(),
            TextColumn("{task.fields[eta]}") ,
            transient=True,
        )
        prog.start()
        task_id = prog.add_task("ncd", total=len(jobs), eta="")

    try:
        with out.open("w", encoding="utf-8") as f:
            for k, job in enumerate(jobs, start=1):
                x = payloads[job.a.key]
                y = payloads[job.b.key]

                # Time the full NCD computation; treat input bytes as |x|+|y|.
                # (Compression is called 3x inside compute_ncd; this estimate is coarse.)
                t0 = time.perf_counter()
                res = compute_ncd(compressor=comp, x=x, y=y)
                t1 = time.perf_counter()

                est.add_sample(TimingSample(input_bytes=job.input_bytes, wall_time_s=t1 - t0))
                if est.should_refit():
                    est.fit_from_first_n(6)

                row = {
                    "a": job.a.key,
                    "b": job.b.key,
                    "a_bytes": len(x),
                    "b_bytes": len(y),
                    "c_x": res.c_x,
                    "c_y": res.c_y,
                    "c_xy": res.c_xy,
                    "ncd": res.ncd,
                }
                f.write(json.dumps(row) + "\n")

                eta_text = ""
                if show_eta and est.fit and k < len(jobs):
                    rem_s = est.estimate_remaining_s(j.input_bytes for j in jobs[k:])
                    if rem_s is not None:
                        eta_text = f"ETA {_format_seconds(rem_s)} (model={est.fit.model}, r2={est.fit.r2:.2f})"

                if prog is not None and task_id is not None:
                    prog.update(task_id, advance=1, eta=eta_text)
                elif show_eta and est.fit and (k % 25 == 0 or k == len(jobs)):
                    if eta_text:
                        typer.echo(f"[{k}/{len(jobs)}] {eta_text}")

        typer.echo(f"wrote {len(jobs)} rows to {out}")
    finally:
        if prog is not None:
            prog.stop()
