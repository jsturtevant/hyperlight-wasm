# GitHub Copilot Agent + Hyperlight Wasm Sandbox

Run sandboxed Python code in Hyperlight Wasm sandboxes with Microsoft Agent Framework and GitHub Copilot.

## Quick Start

```bash
# From repo root
python3 -m venv .venv && source .venv/bin/activate
pip install agent-framework-github-copilot --pre

gh auth login

# Build the guest AOT module
just guest-build

# Build/install the local Hyperlight Python package
just python-build

just agent-framework-example

# Interactive multi-turn REPL
just agent-framework-example-interactive

# DevUI web interface
pip install agent-framework-devui --pre
just agent-framework-example-devui
```

## How It Works

Copilot sees three tools (`execute_code`, `compute`, `fetch_data`) but the system prompt steers it to write Python code via `execute_code`. Inside the Wasm sandbox, generated code calls host functions through `call_tool()` (a built-in global).

```
Copilot Agent → execute_code(code="...")
                  │
                  ▼
             WasmSandbox.run(code)
                  │
                  ├── call_tool("fetch_data", table="users") → host
                  ├── call_tool("compute", operation="multiply", a=6, b=7) → host
                  ▼
             stdout returned to the agent
```

`compute` and `fetch_data` are exposed to the model for schema guidance, and `_create_sandbox()` registers them as host callbacks so sandboxed code can call them via `call_tool()`.

## Key Differences from hyperlight-unikraft Version

| Aspect | hyperlight-unikraft | hyperlight-wasm |
|--------|--------------------|-----------------| 
| Sandbox | `HyperlightSandbox(kernel_path=..., initrd_path=...)` | `WasmSandbox(module_path=...)` |
| Startup | ~5-10ms (micro-VM boot) | ~1-2ms (AOT Wasm) |
| Guest call_tool | `from hyperlight import call_tool` (required) | Built-in global (no import needed) |
| Result type | Dict with `.get("success")` | Object with `.success`, `.stdout` |
| Env vars | `HYPERLIGHT_KERNEL`, `HYPERLIGHT_INITRD` | `HYPERLIGHT_MODULE` |

## Compare With

- `../python-sdk/basic.py` — Basic WasmSandbox usage without Copilot
- `../copilot-sdk/copilot_sdk_tools.py` — Copilot SDK version (lower-level API)
