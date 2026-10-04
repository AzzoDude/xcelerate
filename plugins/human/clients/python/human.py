"""Client for the human plugin (generated)."""
from __future__ import annotations

import json
from typing import Any, Dict, List, Optional, TypedDict

PLUGIN = "human"

class Info(TypedDict, total=False):
    name: str
    enabled: bool
    ops: List[str]

class Move(TypedDict, total=False):
    moved: bool
    x: float
    y: float

class Click(TypedDict, total=False):
    clicked: bool
    x: float
    y: float

class Type(TypedDict, total=False):
    typed: int

class Scroll(TypedDict, total=False):
    scrolled: float

class Delay(TypedDict, total=False):
    sleptMs: int

class Human:
    def __init__(self, browser: Any):
        self._browser = browser

    async def info(self) -> Info:
        args = {}
        raw = await self._browser.plugin(PLUGIN).invoke("info", json.dumps(args))
        return json.loads(raw)

    async def move(self, x: float, y: float) -> Move:
        args = {"x": x, "y": y}
        raw = await self._browser.plugin(PLUGIN).invoke("move", json.dumps(args))
        return json.loads(raw)

    async def click(self, x: float, y: float) -> Click:
        args = {"x": x, "y": y}
        raw = await self._browser.plugin(PLUGIN).invoke("click", json.dumps(args))
        return json.loads(raw)

    async def type(self, text: str) -> Type:
        args = {"text": text}
        raw = await self._browser.plugin(PLUGIN).invoke("type", json.dumps(args))
        return json.loads(raw)

    async def scroll(self, deltaY: float) -> Scroll:
        args = {"deltaY": deltaY}
        raw = await self._browser.plugin(PLUGIN).invoke("scroll", json.dumps(args))
        return json.loads(raw)

    async def delay(self, minMs: int = None, maxMs: int = None) -> Delay:
        args = {"minMs": minMs, "maxMs": maxMs}
        raw = await self._browser.plugin(PLUGIN).invoke("delay", json.dumps(args))
        return json.loads(raw)
