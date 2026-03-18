#!/usr/bin/env python3
"""GitHub Copilot Agent Framework + Hyperlight Wasm sandbox example.

This example uses Microsoft Agent Framework's GitHubCopilotAgent with three
schema-visible tools:
- execute_code: the primary tool for isolated execution in a Hyperlight Wasm sandbox
- compute/fetch_data: host callbacks whose schemas guide the model

The system prompt steers Copilot to call execute_code and then use
call_tool() inside the sandbox for compute/fetch_data.
"""

import asyncio
import os
import time
from pathlib import Path
from typing import Annotated, Any

from agent_framework.github import GitHubCopilotAgent
from copilot import PermissionHandler
from pydantic import Field

from hyperlight_sandbox import WasmSandbox

SYSTEM_PROMPT = """You have one primary tool: execute_code. It runs Python in an isolated Hyperlight Wasm sandbox.

You do NOT have direct access to any data. The ONLY way to fetch data or perform
computations is by writing Python code via execute_code that calls `call_tool()`
inside the sandbox. NEVER hardcode or assume data values — always call the tool.

`call_tool` is a built-in global inside the sandbox — no import needed.

You MUST use call_tool() for:
  - Fetching data:  call_tool('fetch_data', table='users')
  - Math operations: call_tool('compute', operation='multiply', a=6, b=7)

CORRECT (keyword arguments only):
  users = call_tool('fetch_data', table='users')
  result = call_tool('compute', operation='multiply', a=6, b=7)

WRONG — do NOT pass a dict:
  call_tool('compute', {"operation": "multiply", "a": 6, "b": 7})

WRONG — do NOT hardcode data that should come from call_tool:
  users = [{"name": "Alice"}]  # NEVER do this

The sandbox also has file I/O capabilities:
  - Files pre-loaded by the host are available at /input/<filename>
  - Code can write results to /output/<filename>
  - Attempting to read a file that doesn't exist raises FileNotFoundError

The sandbox has WASI-HTTP networking with an allowlist:
  - `http_get(url)` and `http_post(url, body)` are built-in globals — no import needed
  - They return {"status": int, "body": str}
  - The host controls which domains/paths/methods are allowed
  - Requests to non-allowed destinations raise an error (ErrorCode_HttpRequestDenied)

Do NOT call compute or fetch_data as tools directly. Use execute_code.
Solve each request in a single execute_code call when possible.
Always include the complete stdout from execute_code in your response to the user."""

_SIMULATED_DATA = {
    "users": [
        {"id": 1, "name": "Alice", "role": "admin"},
        {"id": 2, "name": "Bob", "role": "user"},
        {"id": 3, "name": "Charlie", "role": "admin"},
    ],
    "products": [
        {"id": 101, "name": "Widget", "price": 9.99},
        {"id": 102, "name": "Gadget", "price": 19.99},
    ],
}


def compute(
    operation: Annotated[str, Field(description="Math operation: add, subtract, multiply, or divide.")],
    a: Annotated[float, Field(description="First numeric operand.")],
    b: Annotated[float, Field(description="Second numeric operand.")],
) -> float:
    """Perform a math operation."""
    ops = {"add": a + b, "subtract": a - b, "multiply": a * b, "divide": a / b if b else float("inf")}
    return ops.get(operation, 0.0)


def fetch_data(
    table: Annotated[str, Field(description="Name of the simulated table to query.")],
) -> list[dict[str, Any]]:
    """Fetch simulated records from a named table."""
    return _SIMULATED_DATA.get(table, [])


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _default_module_path() -> Path:
    return _repo_root() / "src/python_sandbox/python-sandbox.aot"


# --- Sandbox singleton with snapshot/restore ---
# The sandbox is created once at startup (cold start ~680ms), snapshotted, then
# restored before each execute_code call for clean state with fast startup.

_sandbox = None
_snapshot = None


def _init_sandbox() -> None:
    """Initialize the sandbox and take a snapshot. Call once at program start."""
    global _sandbox, _snapshot

    default_module = _default_module_path()
    module_path = Path(os.environ.get("HYPERLIGHT_MODULE", str(default_module)))

    if not module_path.exists():
        raise RuntimeError(
            "Hyperlight Wasm module not found.\n"
            f"  module: {module_path} (MISSING)\n"
            "Build the python-sandbox AOT module first, or set HYPERLIGHT_MODULE."
        )

    start = time.perf_counter()
    _sandbox = WasmSandbox(module_path=str(module_path))
    _sandbox.register_tool("compute", lambda **kw: compute(**kw))
    _sandbox.register_tool("fetch_data", lambda **kw: fetch_data(**kw))
    _sandbox.add_file("team.json", b'{"members": [{"name": "Alice", "role": "eng"}, {"name": "Bob", "role": "pm"}]}')

    # Network allowlist: only httpbin.org GET, and jsonplaceholder for any method
    _sandbox.allow("https://httpbin.org", methods=["GET"])
    _sandbox.allow("jsonplaceholder.typicode.com")

    # Warm up the sandbox (first run triggers init) and snapshot clean state
    _sandbox.run('None')
    _snapshot = _sandbox.snapshot()
    elapsed_ms = (time.perf_counter() - start) * 1000
    print(f"\U0001f4f8 Sandbox initialized and snapshotted ({elapsed_ms:.0f}ms)")


def _get_sandbox() -> WasmSandbox:
    """Restore sandbox to clean snapshot state and return it."""
    _sandbox.restore(_snapshot)
    return _sandbox


async def execute_code(
    code: Annotated[str, Field(description="Python code to execute in an isolated Hyperlight Wasm sandbox. Use call_tool('fetch_data', table=...) and call_tool('compute', operation=..., a=..., b=...) inside the code to access data and perform calculations. NEVER hardcode data.")],
) -> str:
    """Execute code with snapshot/restore for clean state between calls."""
    try:
        print(f"--- Copilot generated code ---\n{code}\n--- end ---\n")
        sandbox = _get_sandbox()
        start = time.perf_counter()
        result = sandbox.run(code=code)
        elapsed_ms = (time.perf_counter() - start) * 1000
        if result.success:
            stdout = result.stdout.replace("\r\n", "\n")
            print(f"⏱️  execute_code completed ({elapsed_ms:.1f}ms)")
            if not stdout:
                return "Code executed successfully (no output)."
            return (
                "The code ran successfully. Here is the exact output — "
                "include it verbatim in your response:\n\n"
                f"```\n{stdout}\n```"
            )
        stderr = result.stderr or "Unknown error"
        print(f"⏱️  execute_code failed ({elapsed_ms:.1f}ms)")
        return f"Execution error:\n{stderr}"
    except Exception as exc:
        return f"Sandbox error: {exc}"


def create_agent() -> GitHubCopilotAgent:
    return GitHubCopilotAgent(
        name="HyperlightWasmSandbox",
        default_options={
            "instructions": SYSTEM_PROMPT,
            "on_permission_request": PermissionHandler.approve_all,
        },
        tools=[execute_code, compute, fetch_data],
    )


async def main() -> None:
    _init_sandbox()  # pay cold start once, upfront
    agent = create_agent()
    async with agent:
        session = agent.create_session()
        if "--interactive" not in sys.argv:
            prompts = [
                "Fetch all users, find admins, multiply 6*7, and print the users, admins, and multiplication result. Use one execute_code call.",
                "Use execute_code to try reading /input/secrets.txt (it doesn't exist — handle the error), then read /input/team.json which does exist, parse it, and print each team member's name and role.",
                (
                    "Use execute_code to demonstrate the network allowlist. In a single code block:\n"
                    "1. Use http_get to fetch https://httpbin.org/get — this should succeed (GET is allowed)\n"
                    "2. Try http_post to https://httpbin.org/post — this should FAIL (only GET is allowed for httpbin.org)\n"
                    "3. Try http_get to https://example.com — this should FAIL (example.com is not in the allowlist)\n"
                    "4. Use http_get to fetch https://jsonplaceholder.typicode.com/todos/1 — this should succeed (all methods allowed)\n"
                    "Wrap each call in try/except and print whether it succeeded or was blocked."
                ),
            ]
            for i, prompt in enumerate(prompts):
                if i > 0:
                    input("\nPress Enter to continue...")
                    print()
                print(f"User: {prompt}\n")
                result = await agent.run(prompt, session=session)
                print(f"Agent: {result}\n")
            return
        print("Hyperlight Wasm Sandbox Agent (type 'quit' to exit)\n")
        while True:
            try:
                prompt = input("You: ").strip()
            except (EOFError, KeyboardInterrupt):
                break
            if not prompt or prompt.lower() in ("quit", "exit"):
                break
            result = await agent.run(prompt, session=session)
            print(f"Agent: {result}\n")


if __name__ == "__main__":
    import sys
    if "--devui" in sys.argv:
        from agent_framework.devui import serve
        _init_sandbox()  # pay cold start once, upfront
        agent = create_agent()
        serve(entities=[agent], auto_open=True)
    else:
        asyncio.run(main())
