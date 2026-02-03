from __future__ import annotations

import json
import pathlib
from dataclasses import asdict, dataclass
from typing import Any, Optional


@dataclass(frozen=True)
class RunConfig:
    compressor: str = "gzip"
    pairs: str = "upper"
    max_item_bytes: Optional[int] = None


@dataclass(frozen=True)
class GenMatrixConfig:
    rows: int = 8
    cols: int = 8
    min_bytes: int = 512
    max_bytes: int = 8192
    seed: int = 0
    pattern: str = "gradient"


@dataclass(frozen=True)
class AppConfig:
    gen_matrix: GenMatrixConfig = GenMatrixConfig()
    run: RunConfig = RunConfig()


def _load_toml(path: pathlib.Path) -> dict[str, Any]:
    import tomllib

    data: Any = tomllib.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        return {}
    return data


def _load_yaml(path: pathlib.Path) -> dict[str, Any]:
    try:
        import yaml  # type: ignore[import-untyped]
    except Exception as e:  # pragma: no cover
        raise RuntimeError("PyYAML not installed; cannot read YAML config") from e

    data: Any = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        return {}
    return data


def load_config(path: pathlib.Path) -> dict[str, Any]:
    ext = path.suffix.lower()
    if ext in (".toml",):
        return _load_toml(path)
    if ext in (".json",):
        data: Any = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(data, dict):
            return {}
        return data
    if ext in (".yaml", ".yml"):
        return _load_yaml(path)
    raise ValueError(f"Unsupported config extension: {ext}")


def coerce_app_config(data: dict[str, Any]) -> AppConfig:
    gm = data.get("gen_matrix", {}) or {}
    run = data.get("run", {}) or {}

    gen_matrix = GenMatrixConfig(
        rows=int(gm.get("rows", 8)),
        cols=int(gm.get("cols", 8)),
        min_bytes=int(gm.get("min_bytes", 512)),
        max_bytes=int(gm.get("max_bytes", 8192)),
        seed=int(gm.get("seed", 0)),
        pattern=str(gm.get("pattern", "gradient")),
    )
    run_cfg = RunConfig(
        compressor=str(run.get("compressor", "gzip")),
        pairs=str(run.get("pairs", "upper")),
        max_item_bytes=run.get("max_item_bytes", None),
    )
    if run_cfg.max_item_bytes is not None:
        run_cfg = RunConfig(
            compressor=run_cfg.compressor,
            pairs=run_cfg.pairs,
            max_item_bytes=int(run_cfg.max_item_bytes),
        )
    return AppConfig(gen_matrix=gen_matrix, run=run_cfg)


def default_config_dict() -> dict[str, Any]:
    return asdict(AppConfig())
