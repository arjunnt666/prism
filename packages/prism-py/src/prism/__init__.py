"""Prism Python helpers for notebook analysis of snapshot JSON."""
from __future__ import annotations
import json
from pathlib import Path
from typing import Any, Dict, List

def load_snapshot(path: str | Path) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)

def domains(snapshot: Dict[str, Any]) -> List[str]:
    return [r.get("domain", "") for r in snapshot.get("results", [])]

def positions(snapshot: Dict[str, Any]) -> Dict[str, int]:
    out: Dict[str, int] = {}
    for r in snapshot.get("results", []):
        d = r.get("domain", "").lower()
        if d and d not in out:
            out[d] = int(r.get("position", 0))
    return out

def version() -> str:
    return "0.1.0"

__all__ = ["load_snapshot", "domains", "positions", "version"]
