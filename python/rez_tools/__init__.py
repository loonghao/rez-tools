"""
rez-tools: A high-performance suite tool command line for rez

This package provides a Python wrapper around the Rust-based rez-tools binary,
allowing users to install and use rez-tools through pip while benefiting from
the performance of the native Rust implementation.
"""

__version__ = "0.1.0"
__author__ = "Long Hao"
__email__ = "hal.long@outlook.com"

from .core import RezTools, get_binary_path, ensure_binary_installed

__all__ = [
    "RezTools",
    "get_binary_path", 
    "ensure_binary_installed",
    "__version__",
]
