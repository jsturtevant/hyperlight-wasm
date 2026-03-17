# GitHub Copilot SDK + Hyperlight Wasm Sandbox

Run sandboxed Python code in Hyperlight Wasm sandboxes, orchestrated by GitHub Copilot.

## Quick Start

```bash
# From repo root
python3 -m venv .venv && source .venv/bin/activate
pip install github-copilot-sdk pydantic

# Build the guest AOT module
just guest-build

# Build/install the local Hyperlight Python package
just python-build

gh auth login

just copilot-sdk-example
```

## How It Works

Copilot sees three tools (`execute_code`, `compute`, `fetch_data`) but the system prompt steers it to write Python code via `execute_code`. Inside the Wasm sandbox, the generated code calls host functions through `call_tool()` (a built-in global).

```
Copilot → execute_code(code="...")
              │
              ▼
         WasmSandbox.run(code)
              │
              ├── call_tool("fetch_data", table="users")  → host
              ├── call_tool("compute", op="multiply", a=6, b=7)  → host
              │
              ▼
         stdout returned to Copilot
```

### Example Output

```
🔧 [copilot → sdk] execute_code
--- Copilot generated code ---
users = call_tool("fetch_data", table="users")
result = call_tool("compute", operation="multiply", a=6, b=7)
admins = [u for u in users if u.get("role") == "admin"]
print(f"Admins: {[a['name'] for a in admins]}")
print(f"6 * 7 = {result}")
--- end ---

✅ [copilot → sdk] execute_code done
```

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
- `../agent-framework/copilot_agent.py` — Microsoft Agent Framework version
