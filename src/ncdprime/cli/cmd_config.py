from __future__ import annotations

import json
import pathlib

import typer

from ncdprime.core.config import coerce_app_config, default_config_dict, load_config

app = typer.Typer(add_completion=False, no_args_is_help=True)


@app.command("init")
def config_init(
    out: pathlib.Path = typer.Argument(pathlib.Path("ncdprime.toml"), help="Output config path."),
    format: str = typer.Option("toml", "--format", help="toml|json"),
    overwrite: bool = typer.Option(False, "--overwrite"),
) -> None:
    """Write a starter config file."""
    if out.exists() and not overwrite:
        raise typer.BadParameter(f"{out} exists; pass --overwrite")

    data = default_config_dict()

    if format == "json":
        out.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    elif format == "toml":
        # Minimal TOML emitter (enough for our nested dict).
        lines: list[str] = []
        gm = data["gen_matrix"]
        lines += ["[gen_matrix]"]
        for k, v in gm.items():
            lines.append(f"{k} = {json.dumps(v)}")
        run = data["run"]
        lines += ["", "[run]"]
        for k, v in run.items():
            lines.append(f"{k} = {json.dumps(v)}")
        out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    else:
        raise typer.BadParameter("format must be toml or json")

    typer.echo(f"wrote {out}")


@app.command("resolve")
def config_resolve(
    path: pathlib.Path = typer.Argument(..., help="Config file (TOML/JSON/YAML)."),
    out: pathlib.Path = typer.Option(pathlib.Path("ncdprime.lock.json"), "--out"),
) -> None:
    """Resolve a config into a lockfile-style normalized JSON."""
    raw = load_config(path)
    cfg = coerce_app_config(raw)

    from dataclasses import asdict

    out.write_text(json.dumps(asdict(cfg), indent=2) + "\n", encoding="utf-8")
    typer.echo(f"wrote {out}")
