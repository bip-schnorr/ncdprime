import typer

from ncdprime.cli import cmd_compressors, cmd_config, cmd_gen, cmd_run

app = typer.Typer(add_completion=False, no_args_is_help=True)


@app.callback()
def _root(
    config: str = typer.Option(
        None,
        "--config",
        "-c",
        help="Path to config file (TOML/YAML/JSON).",
    ),
    verbose: int = typer.Option(0, "-v", "--verbose", count=True),
    quiet: bool = typer.Option(False, "--quiet"),
) -> None:
    """ncdprime: generate directory matrices, run NCD, and manage compressor plugins."""
    # Note: global config/logging wiring is added in cmd_config + cmd_run.
    _ = (config, verbose, quiet)


app.add_typer(cmd_gen.app, name="gen")
app.add_typer(cmd_run.app, name="run")
app.add_typer(cmd_compressors.app, name="compressors")
app.add_typer(cmd_config.app, name="config")
