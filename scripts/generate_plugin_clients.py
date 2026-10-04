#!/usr/bin/env python3
"""Generate native, idiomatic client libraries for every binding language from a
plugin's ``api.json``.

Write the plugin's API once (``api.json``); every language gets a real library -
a client **object** with **natively named** methods and typed data - that hides
the JSON wire entirely. ``post`` in the schema becomes ``post`` (Python/Ruby),
``post`` (TypeScript/Java/Kotlin/Swift/Dart), ``Post`` (C#/Go), ``Invoke-...``
(PowerShell) - each language's own convention.

Usage::

    python scripts/generate_plugin_clients.py plugins/my-plugin

Writes ``<plugin-dir>/clients/<language>/…``.

Type grammar::

    string | int | float | bool | any      primitives
    T?                                     optional
    [T]                                    list
    map<K, V>                              map
    Name                                   a type from `types`
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

PRIMITIVES = {"string", "int", "float", "bool", "any"}


# --------------------------------------------------------------------------- #
# Naming
# --------------------------------------------------------------------------- #
def _words(name: str) -> list[str]:
    return [w for w in re.split(r"[^0-9A-Za-z]+|(?<=[a-z0-9])(?=[A-Z])", name) if w]


def snake(name: str) -> str:
    return "_".join(w.lower() for w in _words(name))


def camel(name: str) -> str:
    parts = _words(name)
    return parts[0].lower() + "".join(p.capitalize() for p in parts[1:]) if parts else ""


def pascal(name: str) -> str:
    return "".join(w.capitalize() for w in _words(name))


def title(name: str) -> str:
    return pascal(name.split(".")[-1])


def module_name(plugin: str) -> str:
    return snake(plugin.replace(".", "_"))


# --------------------------------------------------------------------------- #
# Type grammar
# --------------------------------------------------------------------------- #
def parse_type(text: str) -> dict:
    text = text.strip()
    if text.endswith("?"):
        return {"kind": "opt", "of": parse_type(text[:-1])}
    if text.startswith("[") and text.endswith("]"):
        return {"kind": "array", "of": parse_type(text[1:-1])}
    if text.startswith("map<") and text.endswith(">"):
        key, value = text[4:-1].split(",", 1)
        return {"kind": "map", "key": parse_type(key), "value": parse_type(value)}
    if text in PRIMITIVES:
        return {"kind": "prim", "name": text}
    return {"kind": "ref", "name": text}


# --------------------------------------------------------------------------- #
# python - object client, snake_case
# --------------------------------------------------------------------------- #
def _py(node: dict) -> str:
    if node["kind"] == "opt":
        return f"Optional[{_py(node['of'])}]"
    if node["kind"] == "prim":
        return {"string": "str", "int": "int", "float": "float", "bool": "bool", "any": "Any"}[node["name"]]
    if node["kind"] == "array":
        return f"List[{_py(node['of'])}]"
    if node["kind"] == "map":
        return f"Dict[{_py(node['key'])}, {_py(node['value'])}]"
    return node["name"]


def emit_python(schema: dict) -> dict:
    cls = title(schema["plugin"])
    out = [
        f'"""Client for the {schema["plugin"]} plugin (generated)."""',
        "from __future__ import annotations",
        "",
        "import json",
        "from typing import Any, Dict, List, Optional, TypedDict",
        "",
        f'PLUGIN = "{schema["plugin"]}"',
        "",
    ]
    for name, spec in schema["types"].items():
        out.append(f"class {name}(TypedDict, total=False):")
        fields = spec.get("fields", {})
        out += [f"    {f}: {_py(parse_type(t))}" for f, t in fields.items()] or ["    pass"]
        out.append("")
    out += [f"class {cls}:", "    def __init__(self, browser: Any):", "        self._browser = browser", ""]
    for op in schema["ops"]:
        params = ", ".join(
            f"{p['name']}: {_py(parse_type(p['type']))} = None" if p.get("optional") else f"{p['name']}: {_py(parse_type(p['type']))}"
            for p in op["params"]
        )
        ret = _py(parse_type(op["result"]))
        if op.get("optional_result"):
            ret = f"Optional[{ret}]"
        sig = f"    async def {op['name']}(self, {params}) -> {ret}:" if params else f"    async def {op['name']}(self) -> {ret}:"
        out.append(sig)
        entries = ", ".join(f'"{p["name"]}": {p["name"]}' for p in op["params"])
        out.append(f"        args = {{{entries}}}")
        out.append(f'        raw = await self._browser.plugin(PLUGIN).invoke("{op["name"]}", json.dumps(args))')
        if op.get("optional_result"):
            out.append('        return None if raw in (None, "null") else json.loads(raw)')
        else:
            out.append("        return json.loads(raw)")
        out.append("")
    return {f"python/{module_name(schema['plugin'])}.py": "\n".join(out)}


# --------------------------------------------------------------------------- #
# typescript - object client, camelCase
# --------------------------------------------------------------------------- #
def _ts(node: dict) -> str:
    if node["kind"] == "opt":
        return f"{_ts(node['of'])} | null"
    if node["kind"] == "prim":
        return {"string": "string", "int": "number", "float": "number", "bool": "boolean", "any": "unknown"}[node["name"]]
    if node["kind"] == "array":
        return f"{_ts(node['of'])}[]"
    if node["kind"] == "map":
        return f"Record<string, {_ts(node['value'])}>"
    return node["name"]


def emit_typescript(schema: dict, javascript: bool = False) -> dict:
    cls = title(schema["plugin"])
    out = [
        f"// Client for the {schema['plugin']} plugin (generated).",
        "/* eslint-disable */",
        "",
        f'export const PLUGIN = "{schema["plugin"]}";',
        "",
    ]
    for name, spec in schema["types"].items():
        out.append(f"export interface {name} {{")
        out += [f"  {f}?: {_ts(parse_type(t))};" for f, t in spec.get("fields", {}).items()]
        out.append("}")
        out.append("")
    out.append(f"export class {cls} {{")
    out.append("  constructor(private readonly browser: any) {}")
    out.append("")
    for op in schema["ops"]:
        method = camel(op["name"])
        params = ", ".join(
            f"{camel(p['name'])}?: {_ts(parse_type(p['type']))}" if p.get("optional") else f"{camel(p['name'])}: {_ts(parse_type(p['type']))}"
            for p in op["params"]
        )
        ret = _ts(parse_type(op["result"]))
        if op.get("optional_result"):
            ret = f"{ret} | null"
        out.append(f"  async {method}({params}): Promise<{ret}> {{")
        entries = ", ".join(f"{camel(p['name'])}" for p in op["params"])
        out.append(f"    const args = {{ {entries} }};")
        out.append(f'    const raw = await this.browser.plugin(PLUGIN).invoke("{op["name"]}", JSON.stringify(args));')
        out.append('    return raw === null || raw === "null" ? null : JSON.parse(raw);' if op.get("optional_result") else "    return JSON.parse(raw);")
        out.append("  }")
        out.append("")
    out.append("}")
    name = "javascript" if javascript else "typescript"
    ext = "js" if javascript else "ts"
    return {f"{name}/{module_name(schema['plugin'])}.{ext}": "\n".join(out)}


# --------------------------------------------------------------------------- #
# c# - object client, PascalCase
# --------------------------------------------------------------------------- #
def _cs(node: dict) -> str:
    if node["kind"] == "opt":
        return f"{_cs(node['of'])}?"
    if node["kind"] == "prim":
        return {"string": "string", "int": "long", "float": "double", "bool": "bool", "any": "object"}[node["name"]]
    if node["kind"] == "array":
        return f"List<{_cs(node['of'])}>"
    if node["kind"] == "map":
        return f"Dictionary<{_cs(node['key'])}, {_cs(node['value'])}>"
    return node["name"]


def emit_csharp(schema: dict) -> dict:
    cls = f"{title(schema['plugin'])}Client"
    out = [
        f"// Client for the {schema['plugin']} plugin (generated).",
        "using System.Collections.Generic;",
        "using System.Text.Json;",
        "using System.Text.Json.Serialization;",
        "",
        f"namespace {title(schema['plugin'])};",
        "",
    ]
    for name, spec in schema["types"].items():
        out.append(f"public sealed record {name}")
        out.append("{")
        for f, t in spec.get("fields", {}).items():
            out.append(f'    [JsonPropertyName("{f}")]')
            out.append(f"    public {_cs(parse_type(t))} {pascal(f)} {{ get; init; }}")
        out.append("}")
        out.append("")
    out.append(f"public sealed class {cls}")
    out.append("{")
    out.append(f'    public const string Plugin = "{schema["plugin"]}";')
    out.append("    static readonly JsonSerializerOptions Options = new()")
    out.append("    {")
    out.append("        // The wire keys are exactly the field names; no renaming policy.")
    out.append("        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,")
    out.append("    };")
    out.append("    readonly Browser _browser;")
    out.append(f"    public {cls}(Browser browser) => _browser = browser;")
    out.append("")
    for op in schema["ops"]:
        params = ", ".join(
            f"{_cs(parse_type(p['type']))}? {camel(p['name'])} = null" if p.get("optional") else f"{_cs(parse_type(p['type']))} {camel(p['name'])}"
            for p in op["params"]
        )
        ret = _cs(parse_type(op["result"]))
        if op.get("optional_result"):
            ret = f"{ret}?"
        comma = ", " if params else ""
        out.append(f"    public async Task<{ret}> {pascal(op['name'])}Async({params}{')'}")
        out.append("    {")
        out.append(f'        var raw = await _browser.Plugin(Plugin).Invoke("{op["name"]}", {_cs_args_expr(op)});')
        if op.get("optional_result"):
            out.append(f'        return raw is null or "null" ? null : JsonSerializer.Deserialize<{_cs(parse_type(op["result"]))}>(raw, Options);')
        else:
            out.append(f"        return JsonSerializer.Deserialize<{ret}>(raw, Options)!;")
        out.append("    }")
        out.append("")
    out.append("}")
    return {f"csharp/{cls}.cs": "\n".join(out)}


def _cs_args_expr(op: dict) -> str:
    if not op["params"]:
        return "JsonSerializer.Serialize(new { }, Options)"
    entries = ", ".join(camel(p["name"]) for p in op["params"])
    return f"JsonSerializer.Serialize(new {{ {entries} }}, Options)"


# --------------------------------------------------------------------------- #
# java - object client, camelCase (Gson)
# --------------------------------------------------------------------------- #
def _java(node: dict) -> str:
    if node["kind"] == "opt":
        return _java(node["of"])
    if node["kind"] == "prim":
        return {"string": "String", "int": "long", "float": "double", "bool": "boolean", "any": "Object"}[node["name"]]
    if node["kind"] == "array":
        return f"List<{_java(node['of'])}>"
    if node["kind"] == "map":
        return f"Map<String, {_java(node['value'])}>"
    return node["name"]


def emit_java(schema: dict) -> dict:
    cls = title(schema["plugin"])
    out = [
        f"// Client for the {schema['plugin']} plugin (generated).",
        "package xcelerate.plugins;",
        "",
        "import java.util.LinkedHashMap;",
        "import java.util.List;",
        "import java.util.Map;",
        "import java.util.concurrent.CompletableFuture;",
        "import com.google.gson.Gson;",
        "import com.google.gson.annotations.SerializedName;",
        "import uniffi.xcelerate.Browser;",
        "",
    ]
    for name, spec in schema["types"].items():
        comps = ", ".join(
            f'@SerializedName("{f}") {_java(parse_type(t))} {camel(f)}'
            for f, t in spec.get("fields", {}).items()
        )
        out.append(f"public record {name}({comps}) {{}}")
        out.append("")
    out.append(f"public final class {cls} {{")
    out.append(f'    public static final String PLUGIN = "{schema["plugin"]}";')
    out.append("    private static final Gson GSON = new Gson();")
    out.append("    private final Browser browser;")
    out.append(f"    public {cls}(Browser browser) {{ this.browser = browser; }}")
    out.append("")
    for op in schema["ops"]:
        params = ", ".join(f"{_java(parse_type(p['type']))} {camel(p['name'])}" for p in op["params"])
        ret = _java(parse_type(op["result"]))
        out.append(f"    public CompletableFuture<{ret}> {camel(op['name'])}({params}) {{")
        out.append("        Map<String, Object> args = new LinkedHashMap<>();")
        for p in op["params"]:
            out.append(f'        args.put("{p["name"]}", {camel(p["name"])});')
        out.append("        String json = GSON.toJson(args);")
        if op.get("optional_result"):
            out.append(f'        return browser.plugin(PLUGIN).invoke("{op["name"]}", json)')
            out.append(f'            .thenApply(raw -> raw == null || raw.equals("null") ? null : GSON.fromJson(raw, {ret}.class));')
        else:
            out.append(f'        return browser.plugin(PLUGIN).invoke("{op["name"]}", json)')
            out.append(f'            .thenApply(raw -> GSON.fromJson(raw, {ret}.class));')
        out.append("    }")
        out.append("")
    out.append("}")
    return {f"java/{cls}.java": "\n".join(out)}


# --------------------------------------------------------------------------- #
# kotlin - object client, camelCase (Gson)
# --------------------------------------------------------------------------- #
def _kt(node: dict) -> str:
    if node["kind"] == "opt":
        return _kt(node["of"])
    if node["kind"] == "prim":
        return {"string": "String", "int": "Long", "float": "Double", "bool": "Boolean", "any": "Any"}[node["name"]]
    if node["kind"] == "array":
        return f"List<{_kt(node['of'])}>"
    if node["kind"] == "map":
        return f"Map<String, {_kt(node['value'])}>"
    return node["name"]


def emit_kotlin(schema: dict) -> dict:
    cls = title(schema["plugin"])
    out = [
        f"// Client for the {schema['plugin']} plugin (generated).",
        "package xcelerate.plugins",
        "",
        "import com.google.gson.Gson",
        "import com.google.gson.annotations.SerializedName",
        "import uniffi.xcelerate.Browser",
        "",
    ]
    for name, spec in schema["types"].items():
        out.append(f"data class {name}(")
        for f, t in spec.get("fields", {}).items():
            out.append(f'    @SerializedName("{f}") val {camel(f)}: {_kt(parse_type(t))}? = null,')
        out.append(")")
        out.append("")
    out.append(f"class {cls}(private val browser: Browser) {{")
    out.append("    companion object {")
    out.append(f'        const val PLUGIN = "{schema["plugin"]}"')
    out.append("        private val GSON = Gson()")
    out.append("    }")
    out.append("")
    for op in schema["ops"]:
        params = ", ".join(f"{camel(p['name'])}: {_kt(parse_type(p['type']))}?" for p in op["params"])
        ret = _kt(parse_type(op["result"])) + ("?" if op.get("optional_result") else "")
        out.append(f"    suspend fun {camel(op['name'])}({params}): {ret} {{")
        if op["params"]:
            entries = ", ".join(f'"{p["name"]}" to {camel(p["name"])}' for p in op["params"])
            out.append(f"        val json = GSON.toJson(mapOf({entries}))")
        else:
            out.append('        val json = "{}"')
        if op.get("optional_result"):
            out.append(f'        val raw = browser.plugin(PLUGIN).invoke("{op["name"]}", json)')
            out.append(f'        return if (raw == null || raw == "null") null else GSON.fromJson(raw, {_kt(parse_type(op["result"]))}::class.java)')
        else:
            out.append(f'        return GSON.fromJson(browser.plugin(PLUGIN).invoke("{op["name"]}", json), {ret}::class.java)')
        out.append("    }")
        out.append("")
    out.append("}")
    return {f"kotlin/{cls}.kt": "\n".join(out)}


# --------------------------------------------------------------------------- #
# go - object client, PascalCase
# --------------------------------------------------------------------------- #
def _go(node: dict) -> str:
    if node["kind"] == "opt":
        return "*" + _go(node["of"])
    if node["kind"] == "prim":
        return {"string": "string", "int": "int64", "float": "float64", "bool": "bool", "any": "any"}[node["name"]]
    if node["kind"] == "array":
        return "[]" + _go(node["of"])
    if node["kind"] == "map":
        return f"map[{_go(node['key'])}]{_go(node['value'])}"
    return node["name"]


def emit_go(schema: dict) -> dict:
    pkg = module_name(schema["plugin"])
    out = [
        f"// Client for the {schema['plugin']} plugin (generated).",
        f"package {pkg}",
        "",
        'import "encoding/json"',
        "",
        f'const Plugin = "{schema["plugin"]}"',
        "",
        "// Browser is the part of the engine binding this client needs.",
        "type Browser interface { Plugin(name string) PluginHandle }",
        "type PluginHandle interface { Invoke(op, argsJSON string) (string, error) }",
        "",
    ]
    for name, spec in schema["types"].items():
        out.append(f"type {name} struct {{")
        out += [f'    {pascal(f)} {_go(parse_type(t))} `json:"{f},omitempty"`' for f, t in spec.get("fields", {}).items()]
        out.append("}")
        out.append("")
    out += [
        f"type {title(schema['plugin'])} struct {{ browser Browser }}",
        f"func New(browser Browser) *{title(schema['plugin'])} {{ return &{title(schema['plugin'])}{{browser}} }}",
        "",
    ]
    for op in schema["ops"]:
        params = ", ".join(
            f"{camel(p['name'])} {_go(parse_type(p['type']))}" for p in op["params"]
        )
        ret = _go(parse_type(op["result"]))
        if op.get("optional_result"):
            ret = "*" + _go(parse_type(op["result"]))
        comma = ", " if params else ""
        out.append(f"func (c *{title(schema['plugin'])}) {pascal(op['name'])}({params}) ({ret}, error) {{")
        entries = ", ".join(f'"{p["name"]}": {camel(p["name"])}' for p in op["params"])
        out.append(f"    args, err := json.Marshal(map[string]any{{{entries}}})")
        out.append("    if err != nil { return " + ("nil" if op.get("optional_result") else f"*new({ret})") + ", err }")
        out.append(f'    raw, err := c.browser.Plugin(Plugin).Invoke("{op["name"]}", string(args))')
        out.append("    if err != nil { return " + ("nil" if op.get("optional_result") else f"*new({ret})") + ", err }")
        if op.get("optional_result"):
            out.append('    if raw == "null" || raw == "" { return nil, nil }')
        out.append(f"    var out {_go(parse_type(op['result']))}")
        out.append("    err = json.Unmarshal([]byte(raw), &out)")
        out.append("    return " + ("&out" if op.get("optional_result") else "out") + ", err")
        out.append("}")
        out.append("")
    return {f"go/{pkg}/{pkg}.go": "\n".join(out)}


# --------------------------------------------------------------------------- #
# swift - object client, camelCase (Codable)
# --------------------------------------------------------------------------- #
def _swift(node: dict) -> str:
    if node["kind"] == "opt":
        return f"{_swift(node['of'])}?"
    if node["kind"] == "prim":
        return {"string": "String", "int": "Int", "float": "Double", "bool": "Bool", "any": "AnyCodable"}[node["name"]]
    if node["kind"] == "array":
        return f"[{_swift(node['of'])}]"
    if node["kind"] == "map":
        return f"[String: {_swift(node['value'])}]"
    return node["name"]


def emit_swift(schema: dict) -> dict:
    cls = title(schema["plugin"])
    out = [
        f"// Client for the {schema['plugin']} plugin (generated).",
        "import Foundation",
        "",
        f'public let pluginName = "{schema["plugin"]}"',
        "",
    ]
    for name, spec in schema["types"].items():
        fields = spec.get("fields", {})
        out.append(f"public struct {name}: Codable, Sendable {{")
        for f, t in fields.items():
            ftype = _swift(parse_type(t))
            if not ftype.endswith("?"):
                ftype += "?"
            out.append(f"    public var {camel(f)}: {ftype}")
        out.append("    enum CodingKeys: String, CodingKey {")
        for f in fields:
            out.append(f'        case {camel(f)} = "{f}"')
        out.append("    }")
        out.append("}")
        out.append("")
    for op in schema["ops"]:
        out.append(f"struct {pascal(op['name'])}Request: Encodable {{")
        for p in op["params"]:
            ftype = _swift(parse_type(p["type"]))
            if p.get("optional") and not ftype.endswith("?"):
                ftype += "?"
            out.append(f"    let {camel(p['name'])}: {ftype}")
        out.append("}")
        out.append("")
    out.append(f"public struct {cls} {{")
    out.append("    let browser: Browser")
    out.append("    public init(browser: Browser) { self.browser = browser }")
    out.append("")
    for op in schema["ops"]:
        params = ", ".join(
            f"{camel(p['name'])}: {_swift(parse_type(p['type']))}" + ("? = nil" if p.get("optional") and not _swift(parse_type(p["type"])).endswith("?") else (" = nil" if p.get("optional") else ""))
            for p in op["params"]
        )
        ret = _swift(parse_type(op["result"]))
        if op.get("optional_result"):
            ret = f"{ret}?"
        out.append(f"    public func {camel(op['name'])}({params}) async throws -> {ret} {{")
        argvals = ", ".join(f"{camel(p['name'])}: {camel(p['name'])}" for p in op["params"])
        out.append(f"        let json = String(data: try JSONEncoder().encode({pascal(op['name'])}Request({argvals})), encoding: .utf8)!")
        out.append(f'        let raw = try await browser.plugin(name: pluginName).invoke(op: "{op["name"]}", argsJson: json)')
        out.append(f"        return try JSONDecoder().decode({ret.rstrip('?')}.self, from: Data(raw.utf8))")
        out.append("    }")
        out.append("")
    out.append("}")
    return {f"swift/{cls}.swift": "\n".join(out)}


# --------------------------------------------------------------------------- #
# ruby - object client, snake_case
# --------------------------------------------------------------------------- #
def emit_ruby(schema: dict) -> dict:
    cls = title(schema["plugin"])
    out = [
        f"# Client for the {schema['plugin']} plugin (generated).",
        "# frozen_string_literal: true",
        "",
        'require "json"',
        "",
        f"class {cls}",
        f'  PLUGIN = "{schema["plugin"]}"',
        "  def initialize(browser)",
        "    @browser = browser",
        "  end",
        "",
    ]
    for op in schema["ops"]:
        params = ", ".join((f"{p['name']} = nil" if p.get("optional") else p["name"]) for p in op["params"])
        comma = ", " if params else ""
        out.append(f"  def {op['name']}({params})")
        entries = ", ".join(f'"{p["name"]}" => {p["name"]}' for p in op["params"])
        out.append(f"    args = {{{entries}}}")
        out.append(f'    raw = @browser.plugin(PLUGIN).invoke("{op["name"]}", JSON.generate(args))')
        out.append("    raw.nil? || raw == \"null\" ? nil : JSON.parse(raw)")
        out.append("  end")
        out.append("")
    out.append("end")
    return {f"ruby/{module_name(schema['plugin'])}.rb": "\n".join(out)}


# --------------------------------------------------------------------------- #
# dart - object client, camelCase
# --------------------------------------------------------------------------- #
def _dart(node: dict) -> str:
    if node["kind"] == "opt":
        return f"{_dart(node['of'])}?"
    if node["kind"] == "prim":
        return {"string": "String", "int": "int", "float": "double", "bool": "bool", "any": "Object?"}[node["name"]]
    if node["kind"] == "array":
        return f"List<{_dart(node['of'])}>"
    if node["kind"] == "map":
        return f"Map<String, {_dart(node['value'])}>"
    return node["name"]


def _dart_n(node: dict) -> str:
    text = _dart(node)
    return text if text.endswith("?") else text + "?"


def emit_dart(schema: dict) -> dict:
    cls = title(schema["plugin"])
    objects = set(schema["types"].keys())
    out = [
        f"// Client for the {schema['plugin']} plugin (generated).",
        "import 'dart:convert';",
        "",
        f"const pluginName = '{schema['plugin']}';",
        "",
    ]
    for name, spec in schema["types"].items():
        fields = spec.get("fields", {})
        out.append(f"class {name} {{")
        out += [f"  final {_dart_n(parse_type(t))} {camel(f)};" for f, t in fields.items()]
        ctor = ", ".join(f"this.{camel(f)}" for f in fields)
        out.append(f"  {name}({{{ctor}}});")
        pairs = ", ".join(f"'{f}': {camel(f)}" for f in fields)
        out.append(f"  Map<String, dynamic> toJson() => {{{pairs}}};")
        assigns = ", ".join(f"{camel(f)}: json['{f}'] as {_dart_n(parse_type(t))}" for f, t in fields.items())
        out.append(f"  factory {name}.fromJson(Map<String, dynamic> json) => {name}({assigns});")
        out.append("}")
        out.append("")
    out.append(f"class {cls} {{")
    out.append("  final dynamic browser;")
    out.append(f"  {cls}(this.browser);")
    out.append("")
    for op in schema["ops"]:
        required = [p for p in op["params"] if not p.get("optional")]
        optional = [p for p in op["params"] if p.get("optional")]
        parts = [f"{_dart(parse_type(p['type']))} {camel(p['name'])}" for p in required]
        if optional:
            parts.append("{ " + ", ".join(f"{_dart(parse_type(p['type']))}? {camel(p['name'])}" for p in optional) + " }")
        ret = _dart(parse_type(op["result"]))
        if op.get("optional_result"):
            ret = f"{ret}?"
        out.append(f"  Future<{ret}> {camel(op['name'])}({', '.join(parts)}) async {{")
        entries = []
        for p in op["params"]:
            node = parse_type(p["type"])
            value = f"{camel(p['name'])}.toJson()" if node.get("kind") == "ref" and node["name"] in objects else camel(p["name"])
            if p.get("optional"):
                entries.append(f"if ({camel(p['name'])} != null) '{p['name']}': {value}")
            else:
                entries.append(f"'{p['name']}': {value}")
        out.append(f"    final args = <String, dynamic>{{{', '.join(entries)}}};")
        out.append(f"    final raw = await browser.plugin(pluginName).invoke('{op['name']}', jsonEncode(args));")
        node = parse_type(op["result"])
        if node.get("kind") == "ref" and node["name"] in objects:
            if op.get("optional_result"):
                out.append("    if (raw == null) return null;")
            out.append(f"    return {node['name']}.fromJson(jsonDecode(raw) as Map<String, dynamic>);")
        else:
            out.append(f"    return jsonDecode(raw) as {ret.rstrip('?')};")
        out.append("  }")
        out.append("")
    out.append("}")
    return {f"dart/{module_name(schema['plugin'])}.dart": "\n".join(out)}


# --------------------------------------------------------------------------- #
# powershell - functions, Verb-Noun
# --------------------------------------------------------------------------- #
def emit_powershell(schema: dict) -> dict:
    out = [
        f"# Client for the {schema['plugin']} plugin (generated).",
        "",
        f'$script:Plugin = "{schema["plugin"]}"',
        "",
    ]
    for op in schema["ops"]:
        fn = f"Invoke-{title(schema['plugin'])}{pascal(op['name'])}"
        out.append(f"function {fn} {{")
        out.append("    param(")
        parts = ["[Parameter(Mandatory)] $Browser"] + [f"${pascal(p['name'])}" for p in op["params"]]
        out.append(",\n".join("        " + p for p in parts))
        out.append("    )")
        entries = ", ".join(f'"{p["name"]}" = ${pascal(p["name"])}' for p in op["params"])
        out.append(f"    $args = @{{{entries}}}")
        out.append(f'    $raw = (Get-XceleratePlugin -Browser $Browser -Name $script:Plugin).Invoke("{op["name"]}", ($args | ConvertTo-Json -Compress))')
        out.append("    if ([string]::IsNullOrWhiteSpace($raw) -or $raw -eq 'null') { return $null }")
        out.append("    $raw | ConvertFrom-Json")
        out.append("}")
        out.append("")
    return {f"powershell/{title(schema['plugin'])}.ps1": "\n".join(out)}


EMITTERS = {
    "python": emit_python,
    "typescript": emit_typescript,
    "javascript": lambda s: emit_typescript(s, javascript=True),
    "csharp": emit_csharp,
    "java": emit_java,
    "kotlin": emit_kotlin,
    "go": emit_go,
    "swift": emit_swift,
    "ruby": emit_ruby,
    "dart": emit_dart,
    "powershell": emit_powershell,
}


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate native plugin clients.")
    parser.add_argument("plugin_dir", type=Path)
    parser.add_argument("--out", type=Path, default=None)
    parser.add_argument("--languages", default=",".join(EMITTERS))
    args = parser.parse_args()

    schema = json.loads((args.plugin_dir / "api.json").read_text(encoding="utf-8"))
    out = args.out or (args.plugin_dir / "clients")

    for language in [x.strip() for x in args.languages.split(",") if x.strip()]:
        emitter = EMITTERS.get(language)
        if emitter is None:
            print(f"skip: no generator for '{language}'")
            continue
        for relative, content in emitter(schema).items():
            path = out / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
            print(f"wrote {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
