from __future__ import annotations

import json
from pathlib import Path

from typer.testing import CliRunner

from ncdprime.cli.main import app


def test_gen_matrix_creates_expected_layout(tmp_path: Path) -> None:
    outdir = tmp_path / "m"
    r = CliRunner().invoke(
        app,
        [
            "gen",
            "matrix",
            str(outdir),
            "--rows",
            "2",
            "--cols",
            "3",
            "--min-bytes",
            "10",
            "--max-bytes",
            "10",
            "--seed",
            "123",
        ],
    )
    assert r.exit_code == 0, r.stdout
    assert (outdir / "matrix.json").exists()
    meta = json.loads((outdir / "matrix.json").read_text())
    assert meta["spec"]["rows"] == 2
    assert meta["spec"]["cols"] == 3

    # One payload per cell.
    for cell in meta["cells"]:
        name = f"r{cell['row']:03d}_c{cell['col']:03d}"
        p = outdir / "cells" / name / "payload.bin"
        assert p.exists()
        assert p.stat().st_size == 10


def test_compressors_list_includes_gzip() -> None:
    r = CliRunner().invoke(app, ["compressors", "list"])
    assert r.exit_code == 0
    assert "gzip" in r.stdout


def test_run_matrix_writes_results(tmp_path: Path) -> None:
    outdir = tmp_path / "m"
    out = tmp_path / "results.jsonl"

    r1 = CliRunner().invoke(app, ["gen", "matrix", str(outdir), "--rows", "2", "--cols", "2"])
    assert r1.exit_code == 0, r1.stdout

    r2 = CliRunner().invoke(
        app,
        [
            "run",
            "matrix",
            str(outdir),
            "--compressor",
            "gzip",
            "--out",
            str(out),
            "--no-eta",
        ],
    )
    assert r2.exit_code == 0, r2.stdout
    lines = out.read_text().strip().splitlines()

    # 4 items => upper pairs including diagonal = 4*5/2 = 10
    assert len(lines) == 10
    row0 = json.loads(lines[0])
    assert {"a", "b", "ncd"}.issubset(row0.keys())
