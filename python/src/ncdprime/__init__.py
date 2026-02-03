"""ncdprime: NCD utility with matrix generation and compressor plugins."""

from .estimator import CompletionTimeEstimator, TimingSample

__all__ = [
    "CompletionTimeEstimator",
    "TimingSample",
]

__all__ = ["__version__"]
__version__ = "0.1.0"
