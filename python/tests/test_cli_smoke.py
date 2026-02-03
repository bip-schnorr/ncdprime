from typer.testing import CliRunner

from ncdprime.cli.main import app


def test_cli_help_smoke() -> None:
    r = CliRunner().invoke(app, ["--help"])
    assert r.exit_code == 0
    assert "ncdprime" in r.stdout
