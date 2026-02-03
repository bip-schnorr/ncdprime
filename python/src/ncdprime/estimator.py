from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Iterable, List, Optional, Sequence, Tuple


@dataclass(frozen=True)
class TimingSample:
    """One benchmark observation.

    Attributes:
        input_bytes: Uncompressed size of the item(s) being compressed.
        wall_time_s: Wall clock time spent compressing (seconds).
        output_bytes: Optional compressed size.
    """

    input_bytes: int
    wall_time_s: float
    output_bytes: Optional[int] = None


@dataclass(frozen=True)
class FitResult:
    model: str
    params: Tuple[float, ...]
    r2: float


def _mean(xs: Sequence[float]) -> float:
    return sum(xs) / len(xs)


def _r2(y: Sequence[float], yhat: Sequence[float]) -> float:
    ybar = _mean(y)
    ss_tot = sum((yi - ybar) ** 2 for yi in y)
    ss_res = sum((yi - yhi) ** 2 for yi, yhi in zip(y, yhat))
    if ss_tot <= 0:
        return 0.0
    return max(0.0, 1.0 - (ss_res / ss_tot))


def _fit_linear(x: Sequence[float], y: Sequence[float]) -> FitResult:
    """Fit y ≈ a + b x by least squares."""
    xbar = _mean(x)
    ybar = _mean(y)

    sxx = sum((xi - xbar) ** 2 for xi in x)
    if sxx <= 0:
        # Degenerate: all x equal.
        a, b = ybar, 0.0
    else:
        sxy = sum((xi - xbar) * (yi - ybar) for xi, yi in zip(x, y))
        b = sxy / sxx
        a = ybar - b * xbar

    yhat = [a + b * xi for xi in x]
    return FitResult(model="linear", params=(a, b), r2=_r2(y, yhat))


def _fit_power(x: Sequence[float], y: Sequence[float]) -> Optional[FitResult]:
    """Fit y ≈ k * x^p in log space.

    Returns None if invalid (non-positive x or y).
    """
    if any(xi <= 0 for xi in x) or any(yi <= 0 for yi in y):
        return None

    lx = [math.log(xi) for xi in x]
    ly = [math.log(yi) for yi in y]

    fit = _fit_linear(lx, ly)  # ly ≈ A + p*lx
    A, p = fit.params
    k = math.exp(A)
    yhat = [k * (xi ** p) for xi in x]
    return FitResult(model="power", params=(k, p), r2=_r2(y, yhat))


class CompletionTimeEstimator:
    """Estimate remaining wall clock time based on early timing samples.

    Intended usage:
      - collect a few early samples (e.g. first ~6 cells)
      - fit a smooth model relating input bytes -> compression time
      - use it to estimate total/remaining time for the rest of a 1-D or 2-D run
      - optionally refit later (e.g. ~15 samples) to improve accuracy

    Model choice:
      - tries both linear (a + b*n) and power-law (k*n^p)
      - picks the model with better R^2 on the samples
      - falls back to linear if power-law fit is invalid

    Notes:
      - This estimates compression time only. If your pipeline has overhead
        (I/O, hashing, scheduling, decompression), include those in the timing.
    """

    def __init__(self) -> None:
        self._samples: List[TimingSample] = []
        self._fit: Optional[FitResult] = None

    @property
    def samples(self) -> Tuple[TimingSample, ...]:
        return tuple(self._samples)

    @property
    def fit(self) -> Optional[FitResult]:
        return self._fit

    def add_sample(self, sample: TimingSample) -> None:
        if sample.input_bytes <= 0 or not math.isfinite(sample.wall_time_s) or sample.wall_time_s < 0:
            return
        self._samples.append(sample)

    def fit_from_first_n(self, n: int = 6) -> Optional[FitResult]:
        samples = [s for s in self._samples if s.input_bytes > 0 and s.wall_time_s > 0]
        if len(samples) < 2:
            self._fit = None
            return None

        samples = samples[: max(2, n)]
        x = [float(s.input_bytes) for s in samples]
        y = [float(s.wall_time_s) for s in samples]

        linear = _fit_linear(x, y)
        power = _fit_power(x, y)

        best = linear
        if power and power.r2 >= linear.r2 + 0.02:
            # Prefer power only if meaningfully better.
            best = power

        self._fit = best
        return best

    def predict_time_s(self, input_bytes: int) -> Optional[float]:
        if not self._fit or input_bytes <= 0:
            return None

        n = float(input_bytes)
        if self._fit.model == "linear":
            a = float(self._fit.params[0])
            b = float(self._fit.params[1])
            # Clamp at 0 to avoid negative predictions.
            return float(max(0.0, a + b * n))

        if self._fit.model == "power":
            k = float(self._fit.params[0])
            p = float(self._fit.params[1])
            return float(max(0.0, k * (n ** p)))

        return None

    def estimate_remaining_s(self, remaining_input_bytes: Iterable[int]) -> Optional[float]:
        if not self._fit:
            return None
        total = 0.0
        any_pred = False
        for b in remaining_input_bytes:
            t = self.predict_time_s(int(b))
            if t is None:
                continue
            any_pred = True
            total += t
        return total if any_pred else None

    def should_refit(self) -> bool:
        """Heuristic: refit around 6 samples and again around ~15 samples."""
        k = len(self._samples)
        return k in (6, 15, 16)
