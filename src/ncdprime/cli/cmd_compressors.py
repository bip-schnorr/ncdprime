from __future__ import annotations

import typer

from ncdprime.plugins.registry import iter_compressor_factories

app = typer.Typer(add_completion=False, no_args_is_help=True)


@app.command("list")
def list_compressors() -> None:
    """List available compressors (built-in + entry points)."""
    for name, factory in iter_compressor_factories():
        src = getattr(factory, "__module__", str(factory))
        typer.echo(f"{name}\t{src}")
