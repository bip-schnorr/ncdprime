from __future__ import annotations

import pathlib

import typer

from ncdprime.core.matrix import MatrixSpec, generate_matrix

app = typer.Typer(add_completion=False, no_args_is_help=True)


@app.command("matrix")
def gen_matrix(
    outdir: pathlib.Path = typer.Argument(..., help="Output directory."),
    rows: int = typer.Option(8, "--rows", min=1, help="Number of rows."),
    cols: int = typer.Option(8, "--cols", min=1, help="Number of cols."),
    min_bytes: int = typer.Option(512, "--min-bytes", min=0, help="Minimum payload bytes."),
    max_bytes: int = typer.Option(8192, "--max-bytes", min=0, help="Maximum payload bytes."),
    seed: int = typer.Option(0, "--seed", help="Deterministic seed for content."),
    pattern: str = typer.Option(
        "gradient", "--pattern", help="Size pattern: gradient|row|col|constant"
    ),
    overwrite: bool = typer.Option(False, "--overwrite", help="Overwrite outdir if exists."),
    dry_run: bool = typer.Option(False, "--dry-run", help="Print plan only; do not write."),
) -> None:
    """Generate a deterministic directory matrix dataset."""
    spec = MatrixSpec(
        rows=rows,
        cols=cols,
        min_bytes=min_bytes,
        max_bytes=max_bytes,
        seed=seed,
        pattern=pattern,
    )
    generate_matrix(outdir=outdir, spec=spec, overwrite=overwrite, dry_run=dry_run)
