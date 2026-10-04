#!/usr/bin/env python3
"""example.echo - a minimal xcelerate plugin (ABI rpc/1).

It is a plain program that speaks line-delimited JSON-RPC on stdin/stdout, so it
would work the same written in any language. Two ops are exposed:

* ``echo`` returns its arguments unchanged.
* ``cookies`` asks the host for the browser cookies through the capability-gated
  ``host.get_cookies`` callback (requires the ``read_cookies`` capability).

Load it with ``Browser.load_plugin(".../examples/echo")``.
"""

import json
import sys

PROTOCOL = {"name": "example.echo", "abi": "rpc/1", "ops": ["echo", "cookies"]}

_next_host_id = iter(range(1_000_000, 2_000_000))


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def host_call(method, params):
    """Call the host and block until it answers (this plugin is single-threaded)."""
    host_id = next(_next_host_id)
    send({"jsonrpc": "2.0", "id": host_id, "method": method, "params": params})
    while True:
        line = sys.stdin.readline()
        if not line:
            raise RuntimeError("host closed the connection")
        reply = json.loads(line)
        if reply.get("id") == host_id:
            if "error" in reply:
                raise RuntimeError(reply["error"].get("message", "host error"))
            return reply.get("result")


def handle(message):
    method = message.get("method")
    if method == "describe":
        send({"jsonrpc": "2.0", "id": message["id"], "result": PROTOCOL})
    elif method == "invoke":
        params = message.get("params") or {}
        op = params.get("op")
        args = params.get("args")
        try:
            if op == "echo":
                result = {"echo": args}
            elif op == "cookies":
                result = host_call("host.get_cookies", {})
            else:
                raise RuntimeError(f"unknown op '{op}'")
            send({"jsonrpc": "2.0", "id": message["id"], "result": result})
        except Exception as error:  # noqa: BLE001 - report any failure to the host
            send({"jsonrpc": "2.0", "id": message["id"], "error": {"message": str(error)}})
    elif method == "shutdown":
        sys.exit(0)


def main():
    while True:
        line = sys.stdin.readline()
        if not line:
            break
        line = line.strip()
        if not line:
            continue
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        handle(message)


if __name__ == "__main__":
    main()
