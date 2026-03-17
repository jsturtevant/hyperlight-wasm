# PRD: `hyperlight_sandbox` — Python SDK for Wasm-Isolated Code Execution

**Status:** Phase 1 Complete
**Author:** Dutch (Lead Architect)
**Requested by:** James Sturtevant
**Date:** 2026-03-16
**Package:** `hyperlight_sandbox`

---

## 1. Overview & Goals

`hyperlight_sandbox` is a Python SDK for executing untrusted code inside hardware-isolated WebAssembly sandboxes powered by [hyperlight-wasm](https://github.com/hyperlight-dev/hyperlight-wasm). It targets LLM agent frameworks that need to run AI-generated code safely, with host tool callbacks, file I/O, and snapshot/restore support.

### Goals

1. **Secure code execution** — Run untrusted Python code inside a Wasm component sandbox with hardware-enforced isolation (hypervisor-backed via hyperlight).
2. **Host tool dispatch** — Allow the host to register Python callables as tools. Guest code invokes them via `call_tool()`, which routes through a WIT-defined `tools.dispatch()` interface.
3. **File I/O via WASI** — Pre-populate input files and retrieve output files through WASI filesystem preopens (`/input/`, `/output/`). Files can be pre-loaded on the sandbox via `add_files()` so they persist across runs.
4. **Snapshot/restore** — Expose hyperlight's snapshot mechanism for Wasm sandbox state capture and restoration, enabling reuse and rollback.
5. **WASI-HTTP networking** — Allow guest code to make outbound HTTP requests to explicitly allowlisted domains via `sandbox.add_network("bing.com")`. Uses WASI-HTTP (`wasi:http/outgoing-handler@0.2.3`). No network access by default — every allowed domain must be explicitly added. **Proven feasible** by [hyperlight-wasm-http-example](https://github.com/hyperlight-dev/hyperlight-wasm-http-example), which implements the full WASI HTTP stack as hyperlight host functions.
5. **Pre-built component** — Ship a ready-to-use `python-sandbox.aot` AOT-compiled Wasm component so users don't need a Wasm toolchain.
6. **Agent framework integration** — Provide a `CodeExecutionTool` high-level API shaped for direct use in agent loops (LangChain, Copilot SDK, etc.).
7. **Persistent file loading** — `sandbox.add_files()` pre-loads files into the sandbox's WASI filesystem so they're available across multiple `run()` calls without re-sending on each invocation.

### Key Value Propositions vs. hyperlight-unikraft

| Property | hyperlight-unikraft | hyperlight_sandbox (this project) |
|----------|--------------------|------------------------------------|
| Isolation | Micro-VM (Unikraft kernel) | Wasm component (WASI) |
| Startup | ~5-10ms | ~1-2ms (AOT Wasm, no kernel boot) |
| Snapshots | Not available | Yes — `snapshot()`/`restore()` |
| File I/O | CPIO initrd injection | WASI filesystem preopens |
| Networking | Not available | WASI-HTTP with domain allowlist |
| Module format | Kernel ELF + initrd | AOT-compiled Wasm component |

---

## 2. Non-Goals (for MVP)

- **Capabilities enforcement system.** Deferred. A middleware hook point (~10 lines) will be added to `ToolRegistry` for future enforcement. No `capabilities=` kwarg in the Python SDK for v1.
- **Tool name discovery.** Guest calls `dispatch()` and gets an error if the tool doesn't exist. No listing/enumeration protocol.
- **Unified cross-backend API.** This SDK has its own natural API. A future `hyperlight-flex` package will provide the unified shim layer across hyperlight-unikraft and hyperlight_sandbox backends.
- **Raw Wasm module support.** Only AOT-compiled Wasm components (not raw `.wasm` modules).
- **JavaScript/other language guests.** MVP supports Python guest only via componentize-py.
- **Multi-language custom guest builds.** componentize-py custom builds only.

---

## 3. Architecture

### Layer Cake

```
┌──────────────────────────────────────────────────────────────────┐
│  Python Application / Agent Framework                            │
│  (LangChain, Copilot SDK, custom agent loops)                    │
├──────────────────────────────────────────────────────────────────┤
│  Layer 4: Python SDK                                             │
│  CodeExecutionTool, SandboxEnvironment, ExecutionResult          │
│  Pure Python dataclasses + high-level API                        │
├──────────────────────────────────────────────────────────────────┤
│  Layer 3: PyO3 Bindings                                          │
│  WasmSandbox pyclass                                             │
│  run(), register_tool(), snapshot(), restore()                   │
│  Built with maturin, exposes Rust host library to Python         │
├──────────────────────────────────────────────────────────────────┤
│  Layer 2: Rust Host Library (new crate: hyperlight-wasm-host)    │
│  ToolRegistry (+ middleware hook), WASI FS setup,                │
│  output capture, timeout, snapshot/restore delegation            │
│  Wraps hyperlight-wasm's SandboxBuilder chain                    │
├──────────────────────────────────────────────────────────────────┤
│  Layer 1: hyperlight-wasm (existing)                             │
│  SandboxBuilder → ProtoWasmSandbox → WasmSandbox                │
│  → LoadedWasmSandbox                                             │
│  Hardware-isolated Wasm execution via hypervisor                 │
├──────────────────────────────────────────────────────────────────┤
│  Guest Wasm Component (python-sandbox.aot)                       │
│  Python interpreter compiled via componentize-py                 │
│  Exports executor.run(), imports tools.dispatch()                │
│  WASI filesystem for file I/O                                    │
└──────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Python code                                      Guest Wasm Component
      │                                                      │
      ▼                                                      │
  WasmSandbox.run(code, inputs, outputs)                     │
      │                                                      │
      ├─── Populate /input/ via WASI preopens ──────────────►│
      │                                                      │
      ├─── Call executor.run(code) ─────────────────────────►│
      │                                                 ┌────┤
      │                                                 │ Python interpreter
      │                                                 │ executes `code`
      │                                                 │    │
      │    ◄── tools.dispatch(name, args_json) ─────────┤    │
      │         │                                       │    │
      │    ToolRegistry.dispatch()                      │    │
      │         │                                       │    │
      │    ────► return result JSON ───────────────────►│    │
      │                                                 │    │
      │                                                 └────┤
      │                                                      │
      │◄─── execution-result { stdout, stderr, exit_code } ──┤
      │                                                      │
      ├─── Read /output/ files via WASI ◄────────────────────┤
      │                                                      │
      ▼
  ExecutionResult
```

---

## 4. WIT Interface Specification

The WIT interface defines the contract between host and guest. It follows the component model pattern established in `src/component_sample/wit/example.wit`.

The WIT evolves across phases. Each phase adds imports to the world.

**Phase 1 WIT** (basic code execution):

```wit
package hyperlight:sandbox;

interface executor {
    record execution-result {
        stdout: string,
        stderr: string,
        exit-code: s32,
    }

    /// Execute Python code and return captured output.
    run: func(code: string) -> execution-result;
}

world python-sandbox {
    export executor;
}
```

**Phase 2 WIT** (adds tool dispatch):

```wit
interface tools {
    /// Dispatch a tool call to the host.
    /// Returns JSON: {"result": ...} or {"error": "..."}.
    dispatch: func(name: string, args-json: string) -> string;
}

world python-sandbox {
    import tools;
    export executor;
}
```

**Phase 3 WIT** (adds WASI filesystem for file I/O):

```wit
world python-sandbox {
    import tools;
    import wasi:io/error@0.2.3;
    import wasi:io/streams@0.2.3;
    import wasi:io/poll@0.2.3;
    import wasi:clocks/monotonic-clock@0.2.3;
    import wasi:filesystem/types@0.2.3;
    import wasi:filesystem/preopens@0.2.3;
    import wasi:cli/stdout@0.2.3;
    import wasi:cli/stderr@0.2.3;
    import wasi:cli/stdin@0.2.3;
    export executor;
}
```

**Phase 3.5 WIT** (adds WASI-HTTP networking — full target world):

```wit
world python-sandbox {
    import tools;
    // WASI I/O foundation
    import wasi:io/error@0.2.3;
    import wasi:io/streams@0.2.3;
    import wasi:io/poll@0.2.3;
    // WASI clocks (required by HTTP timeouts)
    import wasi:clocks/monotonic-clock@0.2.3;
    // WASI filesystem
    import wasi:filesystem/types@0.2.3;
    import wasi:filesystem/preopens@0.2.3;
    // WASI CLI (stdout/stderr capture)
    import wasi:cli/stdout@0.2.3;
    import wasi:cli/stderr@0.2.3;
    import wasi:cli/stdin@0.2.3;
    // WASI HTTP (outbound requests with domain allowlist)
    import wasi:http/types@0.2.3;
    import wasi:http/outgoing-handler@0.2.3;
    // WASI random (used by HTTP internals)
    import wasi:random/random@0.2.3;

    export executor;
}
```

The full WASI dependency chain follows [hyperlight-wasm-http-example](https://github.com/hyperlight-dev/hyperlight-wasm-http-example)'s `hyperlight.wit`.

### Design Decisions

- **`tools.dispatch()` returns `string`, not `result<string, string>`**: Errors are encoded in the JSON response (`{"error": "..."}`) rather than using WIT's `result` type. This simplifies guest-side parsing and matches the hyperlight-unikraft pattern.
- **No tool listing**: Guest calls `dispatch()` blindly. Unknown tools return `{"error": "unknown tool: foo"}`. This keeps the interface minimal.
- **WASI filesystem for I/O, not custom WIT**: File I/O uses standard `wasi:filesystem` imports. The host creates a temporary directory on the host filesystem, populates it with files from `add_files()` and per-run `inputs=`, then maps it as a WASI preopen at `/input/`. For outputs, a separate temp directory is mapped at `/output/`. After execution, the host reads files from the output directory. This avoids a custom data-transfer interface and lets guest code use normal `open()`/`read()`/`write()`. Temp directories are cleaned up when the sandbox is dropped.
- **WASI-HTTP with domain allowlist**: Guest code can make HTTP requests, but ONLY to domains explicitly added via `sandbox.add_network(domain)`. The host implements `wasi:http/outgoing-handler@0.2.3` with a filtering proxy that rejects requests to non-allowlisted domains. No network access by default — zero-trust posture. Implementation can be based on [hyperlight-wasm-http-example](https://github.com/hyperlight-dev/hyperlight-wasm-http-example), which already implements the full WASI HTTP host function stack (io, clocks, streams, types, outgoing-handler) for hyperlight-wasm.
- **Persistent file loading via `add_files()`**: Files added via `sandbox.add_files()` are loaded into the WASI filesystem once and persist across multiple `run()` calls. This is more efficient than passing `inputs=` on every call for static data.
- **File override semantics**: When `run(inputs={...})` provides a file with the same name as a pre-loaded file from `add_files()`, the per-run input wins for that invocation only. After the run, the persistent file is restored. Merge strategy: file-by-file override (not full replacement). Persistent files not mentioned in `inputs=` remain available.
- **Snapshot scope**: Snapshots capture Wasm linear memory (which includes the Python interpreter heap, globals, etc.) via `LoadedWasmSandbox::snapshot()`. Snapshots are in-memory `Arc<Snapshot>` references — not serializable. Snapshots do NOT capture `persistent_files` or `allowed_domains` (those are host-side state). After `restore()`, the Wasm state rewinds but host-side configuration remains unchanged.

---

## 5. Rust Host Library API

New crate: `hyperlight-sandbox` (in workspace at `src/hyperlight_sandbox/`).

> **Implementation note:** The crate uses `hyperlight_component_macro::host_bindgen!` (from the `hyperlight-component-macro` crate, not `hyperlight-wasm-macro`) to generate typed component model bindings. It also requires `hyperlight-host` as a dependency since the macro generates code referencing hyperlight-host types.

### ToolRegistry

Replicates the pattern from hyperlight-unikraft's `host/src/lib.rs`:

```rust
use std::collections::HashMap;
use anyhow::Result;

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value> + Send + Sync>>,
    // Middleware hook point for future capabilities enforcement.
    // Phase 4 will populate this with pre-dispatch checks.
    middleware: Vec<Box<dyn Fn(&str, &serde_json::Value) -> Result<()> + Send + Sync>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            middleware: Vec::new(),
        }
    }

    /// Register a tool handler.
    pub fn register<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(serde_json::Value) -> Result<serde_json::Value> + Send + Sync + 'static,
    {
        self.tools.insert(name.to_string(), Box::new(handler));
    }

    /// Add a middleware hook (for future capabilities enforcement).
    pub fn add_middleware<F>(&mut self, hook: F)
    where
        F: Fn(&str, &serde_json::Value) -> Result<()> + Send + Sync + 'static,
    {
        self.middleware.push(Box::new(hook));
    }

    /// Dispatch a tool call. Runs middleware first, then the handler.
    pub fn dispatch(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        // Run middleware hooks (capabilities check point)
        for hook in &self.middleware {
            hook(name, &args)?;
        }

        let handler = self.tools.get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown tool: {}", name))?;

        handler(args)
    }
}
```

### PythonSandbox (Rust-level)

Wraps the hyperlight-wasm sandbox chain:

```rust
use hyperlight_wasm::{SandboxBuilder, LoadedWasmSandbox};
use std::path::Path;
use std::sync::Arc;

pub struct SandboxConfig {
    pub module_path: String,
    pub heap_size: u64,       // bytes
    pub stack_size: u64,      // bytes (mapped to scratch size)
    pub timeout_secs: Option<u64>,
}

pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub outputs: HashMap<String, Vec<u8>>,
}

pub struct PythonSandbox {
    loaded: LoadedWasmSandbox,
    tools: ToolRegistry,
    /// Pre-loaded files that persist across run() calls
    persistent_files: HashMap<String, Vec<u8>>,
    /// Allowlisted domains for WASI-HTTP outbound requests
    allowed_domains: HashSet<String>,
}

impl PythonSandbox {
    /// Build a new PythonSandbox from config.
    ///
    /// Follows the hyperlight-wasm chain:
    /// SandboxBuilder → ProtoWasmSandbox (register host fns)
    ///   → WasmSandbox (load runtime) → LoadedWasmSandbox (load .aot module)
    pub fn new(config: SandboxConfig) -> Result<Self> {
        let proto = SandboxBuilder::new()
            .with_guest_heap_size(config.heap_size)
            .with_guest_scratch_size(config.stack_size as usize)
            .build()?;

        // Register the tools.dispatch host function
        // (details in implementation — binds ToolRegistry.dispatch to WIT import)

        let wasm_sandbox = proto.load_runtime()?;
        let loaded = wasm_sandbox.load_module(&config.module_path)?;

        Ok(Self {
            loaded,
            tools: ToolRegistry::new(),
        })
    }

    /// Execute code in the sandbox.
    pub fn run(
        &mut self,
        code: &str,
        inputs: &[(String, Vec<u8>)],
        output_names: &[String],
    ) -> Result<ExecutionResult> {
        // 1. Populate /input/ via WASI filesystem preopens
        // 2. Call executor.run(code) via call_guest_function
        // 3. Read /output/ files
        // 4. Return ExecutionResult
        todo!()
    }

    /// Take a snapshot of current sandbox state.
    pub fn snapshot(&mut self) -> Result<Arc<Snapshot>> {
        self.loaded.snapshot()
    }

    /// Restore sandbox to a previous snapshot.
    pub fn restore(&mut self, snapshot: Arc<Snapshot>) -> Result<()> {
        self.loaded.restore(snapshot)
    }

    /// Pre-load a file from bytes into /input/ (persists across run() calls).
    pub fn add_file(&mut self, name: &str, data: Vec<u8>) {
        self.persistent_files.insert(name.to_string(), data);
    }

    /// Pre-load files from the local filesystem into /input/ (persists across run() calls).
    pub fn add_files(&mut self, paths: &[&Path]) -> Result<()> {
        for path in paths {
            let name = path.file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid file path: {:?}", path))?
                .to_string_lossy().to_string();
            let data = std::fs::read(path)?;
            self.persistent_files.insert(name, data);
        }
        Ok(())
    }

    /// Allow the guest to make outbound HTTP requests to the given domain.
    /// No network access is allowed by default — every domain must be explicitly added.
    pub fn add_network(&mut self, domain: &str) {
        self.allowed_domains.insert(domain.to_string());
    }
}
```

### Host Function Registration

The `tools.dispatch` WIT import maps to a host function registered on `ProtoWasmSandbox`:

```rust
// During sandbox construction, before load_runtime():
proto.register_host_function("dispatch", {
    let tools = tools.clone(); // Arc<Mutex<ToolRegistry>>
    move |name: String, args_json: String| -> String {
        let tools = tools.lock().unwrap();
        let args: serde_json::Value = serde_json::from_str(&args_json)
            .unwrap_or(serde_json::Value::Null);
        match tools.dispatch(&name, args) {
            Ok(v) => serde_json::to_string(&serde_json::json!({"result": v})).unwrap(),
            Err(e) => serde_json::to_string(&serde_json::json!({"error": e.to_string()})).unwrap(),
        }
    }
})?;
```

---

## 6. Python SDK API

### Core Classes

```python
from hyperlight_sandbox import WasmSandbox, ExecutionResult

# Basic code execution
sandbox = WasmSandbox(
    module_path="python-sandbox.aot",
    heap_size="512Mi",
    stack_size="8Mi",
    timeout_secs=30,
)
result: ExecutionResult = sandbox.run('print("Hello from Wasm!")')
print(result.stdout)    # "Hello from Wasm!\n"
print(result.exit_code) # 0
```

### File I/O (WASI Filesystem)

```python
# Pre-load files on the sandbox (persist across runs) — varargs
sandbox.add_files("data.json", "config.yaml")  # reads from local FS, maps into /input/

# Or add from bytes
sandbox.add_file("data.json", b'{"key": "value"}')

# Files are available in every run() call
result = sandbox.run(
    code='import json; data = json.load(open("/input/data.json")); ...',
    outputs=["result.json"],
)
print(result.outputs["result.json"])  # bytes

# Per-run inputs still supported (override pre-loaded files for that run)
result = sandbox.run(
    code='...',
    inputs={"data.json": b'{"override": true}'},
)
```

### Networking (WASI-HTTP)

```python
# No network access by default. Explicitly allow domains:
sandbox.add_network("api.bing.com")
sandbox.add_network("api.github.com")

# Guest code can now make HTTP requests to allowed domains only
result = sandbox.run("""
import urllib.request
resp = urllib.request.urlopen("https://api.bing.com/search?q=hello")
print(resp.read().decode())
""")
# Requests to non-allowlisted domains raise an error inside the sandbox
```

### Tool Dispatch

```python
sandbox.register_tool("compute", lambda **kw: kw["a"] * kw["b"])

result = sandbox.run("""
from hyperlight import call_tool
answer = call_tool('compute', a=6, b=7)
print(answer)  # 42
""")
```

### Snapshots (Wasm-Specific)

```python
snapshot = sandbox.snapshot()
result1 = sandbox.run("x = 1; print(x)")
sandbox.restore(snapshot)
result2 = sandbox.run("print(x)")  # NameError — state was rolled back
```

### High-Level Agent Tool

```python
from hyperlight_sandbox import CodeExecutionTool, SandboxEnvironment

tool = CodeExecutionTool(
    environment=SandboxEnvironment(
        module_path="python-sandbox.aot",
        heap_size="512Mi",
    ),
    tools=[compute_fn, fetch_fn],
    timeout=30,
)

# Direct use
result = tool.run(code="print(1+1)")

# Or pass `tool` to an agent framework that expects callables
```

### Class Definitions

```python
from dataclasses import dataclass, field
from typing import Any, Callable

@dataclass
class SandboxEnvironment:
    """Configuration for the Wasm sandbox environment."""
    module_path: str = "python-sandbox.aot"
    heap_size: str = "512Mi"
    stack_size: str = "8Mi"

@dataclass
class ExecutionResult:
    """Result from code execution in a Wasm sandbox."""
    stdout: str
    stderr: str
    exit_code: int
    outputs: dict[str, bytes] = field(default_factory=dict)

    @property
    def success(self) -> bool:
        return self.exit_code == 0

@dataclass
class CodeExecutionTool:
    """High-level tool for agent framework integration.

    Wraps a persistent WasmSandbox instance. The sandbox is created
    once on first use and reused across run() calls. Pre-loaded files
    and registered tools persist. Use snapshot/restore for isolation
    between runs if needed.
    """
    environment: SandboxEnvironment = field(default_factory=SandboxEnvironment)
    tools: list[Callable[..., Any]] = field(default_factory=list)
    timeout: int = 30

    _sandbox: WasmSandbox | None = field(default=None, init=False, repr=False)

    def _get_sandbox(self) -> WasmSandbox:
        """Lazily create and cache the WasmSandbox instance."""
        if self._sandbox is None:
            self._sandbox = WasmSandbox(
                module_path=self.environment.module_path,
                heap_size=self.environment.heap_size,
                stack_size=self.environment.stack_size,
                timeout_secs=self.timeout,
            )
            for tool in self.tools:
                self._sandbox.register_tool(tool)
        return self._sandbox

    def run(
        self,
        code: str,
        inputs: dict[str, bytes] | None = None,
        outputs: list[str] | None = None,
    ) -> ExecutionResult:
        """Execute code in the persistent sandbox."""
        ...
```

### PyO3 Bindings (Layer 3)

The `WasmSandbox` pyclass wraps the Rust `PythonSandbox`. Pattern follows hyperlight-unikraft's `python/src/lib.rs`:

```rust
#[pyclass]
pub struct WasmSandbox {
    inner: PythonSandbox,
    tools: HashMap<String, Py<PyAny>>,  // Python callables
}

#[pymethods]
impl WasmSandbox {
    #[new]
    #[pyo3(signature = (module_path, heap_size="512Mi", stack_size="8Mi", timeout_secs=None))]
    fn new(module_path: &str, heap_size: &str, stack_size: &str, timeout_secs: Option<u64>) -> PyResult<Self> { ... }

    fn run(&self, py: Python, code: &str, inputs: Option<&PyDict>, outputs: Option<&PyList>) -> PyResult<PyDict> { ... }

    /// Register a tool. Accepts either:
    ///   register_tool("name", callable)  — plain Python callable
    ///   register_tool(sdk_tool)           — SDK Tool object with .name and .handler
    #[pyo3(signature = (name_or_tool, callback=None))]
    fn register_tool(&mut self, py: Python, name_or_tool: Py<PyAny>, callback: Option<Py<PyAny>>) -> PyResult<()> { ... }

    fn add_files(&mut self, paths: Vec<String>) -> PyResult<()> { ... }  // Varargs: add_files("f1", "f2")

    fn add_file(&mut self, name: &str, data: &[u8]) -> PyResult<()> { ... }  // Add file from bytes

    fn add_network(&mut self, domain: &str) -> PyResult<()> { ... }  // Allow outbound HTTP to domain

    fn snapshot(&mut self) -> PyResult<PyObject> { ... }   // Returns opaque snapshot handle

    fn restore(&mut self, snapshot: PyObject) -> PyResult<()> { ... }
}
```

---

## 7. Guest Component (componentize-py)

The guest is a Python interpreter compiled to a Wasm component via [componentize-py](https://github.com/bytecodealliance/componentize-py).

### Guest Python Code

The guest component bundles a small Python module that implements the `executor` interface:

```python
# guest/sandbox_executor.py
"""Guest-side executor that runs inside the Wasm component."""

import sys
import io
import json

# Provided by the host via WIT tools.dispatch import
import tools

def run(code: str) -> dict:
    """Execute Python code and capture output."""
    stdout_capture = io.StringIO()
    stderr_capture = io.StringIO()

    old_stdout, old_stderr = sys.stdout, sys.stderr
    sys.stdout = stdout_capture
    sys.stderr = stderr_capture

    exit_code = 0
    try:
        exec(code, {"__builtins__": __builtins__, "call_tool": _call_tool})
    except SystemExit as e:
        exit_code = e.code if isinstance(e.code, int) else 1
    except Exception as e:
        print(f"{type(e).__name__}: {e}", file=stderr_capture)
        exit_code = 1
    finally:
        sys.stdout = old_stdout
        sys.stderr = old_stderr

    return {
        "stdout": stdout_capture.getvalue(),
        "stderr": stderr_capture.getvalue(),
        "exit_code": exit_code,
    }


def _call_tool(name: str, **kwargs) -> any:
    """Call a host-registered tool via WIT dispatch."""
    args_json = json.dumps(kwargs)
    result_json = tools.dispatch(name, args_json)
    result = json.loads(result_json)
    if "error" in result:
        raise RuntimeError(f"Tool '{name}' failed: {result['error']}")
    return result.get("result")
```

### Guest-Side `hyperlight` Module

The guest ships a small `hyperlight` module that user code imports:

```python
# guest/hyperlight.py — available inside the sandbox
"""Hyperlight guest-side helpers, available to user code via `from hyperlight import call_tool`."""

from sandbox_executor import _call_tool as call_tool

__all__ = ["call_tool"]
```

### Building the Guest Component

```bash
# Install componentize-py
pip install componentize-py

# Compile the guest Python code into a Wasm component
componentize-py \
    --wit-path ./wit/hyperlight-sandbox.wit \
    --world python-sandbox \
    componentize \
    -o python-sandbox.wasm \
    sandbox_executor

# AOT-compile for hyperlight-wasm (--component flag for WASI component support)
hyperlight-wasm-aot compile --component python-sandbox.wasm -o python-sandbox.aot
```

The `hyperlight-wasm-aot` CLI is the existing AOT compiler in `src/hyperlight_wasm_aot/`. The `--component` flag is required for WASI components (as demonstrated in [hyperlight-wasm-http-example](https://github.com/hyperlight-dev/hyperlight-wasm-http-example)).

---

## 8. Build & Distribution

### Repository Layout (new files)

```
hyperlight-wasm/
├── src/
│   ├── hyperlight_sandbox/         # Rust host library crate (Phase 1 ✔)
│   │   ├── Cargo.toml
│   │   ├── examples/
│   │   │   └── hello.rs               # Smoke test: runs print('hello') in sandbox
│   │   └── src/
│   │       └── lib.rs                 # PythonSandbox, ToolRegistry, ExecutionResult
│   │
│   └── python_sandbox/            # Guest Wasm component (Phase 1 ✔)
│       ├── wit/
│       │   ├── hyperlight-sandbox.wit # Source WIT
│       │   └── python-sandbox-world.wasm  # Compiled WIT world (for host_bindgen!)
│       ├── sandbox_executor.py    # Guest Python: Executor class with run()
│       └── hyperlight.py          # Guest-side hyperlight module (call_tool stub)
│
├── python/                        # TODO — Pure Python SDK layer
│   ├── hyperlight_sandbox/
│   │   ├── __init__.py            # CodeExecutionTool, SandboxEnvironment, ExecutionResult
│   │   └── py.typed
│   └── pyproject.toml
│
└── assets/                        # TODO — Pre-built artifacts
    └── python-sandbox.aot         # Pre-compiled guest component (CI-built)
```

### Build Steps

1. **Guest component**: `just guest-build` (componentize-py → `.wasm` → `hyperlight-wasm-aot --component` → `.aot`)
2. **Rust host crate**: `cargo build -p hyperlight-sandbox`
3. **Run example**: `just guest-run` (sets `WIT_WORLD` env var automatically)
4. **PyO3 bindings**: `maturin build` (TODO — Phase 4)
5. **Python package**: Standard `pip install .` from `python/` directory (TODO — Phase 4)

> **Critical:** `WIT_WORLD` environment variable must point to `python-sandbox-world.wasm` at runtime. Without it, hyperlight-wasm cannot resolve component export function names. The Justfile `guest-run` target sets this automatically.

### Package Distribution

- **PyPI package name**: `hyperlight-sandbox`
- **Import name**: `hyperlight_sandbox`
- **Includes**: PyO3 native extension + pure Python SDK layer + pre-built `python-sandbox.aot`
- **Build tool**: maturin (same pattern as hyperlight-unikraft's `python/` directory)
- **`.aot` resolution at runtime**: The Python package locates `python-sandbox.aot` in this order:
  1. Explicit `module_path` argument to `WasmSandbox()` (absolute or relative path)
  2. `HYPERLIGHT_MODULE` environment variable
  3. Bundled with the package: `<package_install_dir>/assets/python-sandbox.aot`
  The default `SandboxEnvironment.module_path = "python-sandbox.aot"` triggers the search. An explicit path bypasses it.

---

## 9. Milestones

### Phase 1: Basic Code Execution ✔ COMPLETE

**Goal:** Execute Python code in a Wasm component sandbox, capture stdout/stderr/exit_code.

**Result:** Working end-to-end. `just guest-build && just guest-run` produces:
```
Creating sandbox...
Running print('hello from wasm!')...
exit_code: 0
stdout: "hello from wasm!\n"
stderr: ""
```

**Implementation learnings:**
- componentize-py `--stub-wasi` flag is essential — without it, the guest imports full WASI which hyperlight-wasm can't load
- `componentize-py -d` flag (not `--wit-path`) is the correct syntax
- Guest must define an `Executor` class (not bare functions) matching the WIT interface
- Host uses `hyperlight_component_macro::host_bindgen!` (from `hyperlight-component-macro` crate) to generate typed bindings
- `WIT_WORLD` env var must be set at runtime pointing to `python-sandbox-world.wasm` — hyperlight-wasm uses it to resolve component export names
- `python-sandbox-world.wasm` must be generated from the **original WIT file** (not extracted from the component, which includes componentize-py internal types)
- Guest input buffer size (70MB) must be smaller than heap size — default 8MB heap fails
- Working memory config: heap=200MB, scratch=100MB, input_buffer=70MB
- AOT-compiled Python component is ~43MB (full CPython interpreter)
- `io.StringIO` works in componentize-py ✔ (resolved open question #1)

| Task | Status | Details |
|------|--------|---------|
| Define WIT interface (executor only) | ✔ | `src/python_sandbox/wit/hyperlight-sandbox.wit` |
| Build guest component with componentize-py | ✔ | `just guest-build-wasm` with `--stub-wasi` |
| AOT compile guest | ✔ | `just guest-build-aot` via `hyperlight-wasm-aot compile --component` |
| Create `hyperlight-sandbox` crate | ✔ | `src/hyperlight_sandbox/` with `host_bindgen!` macro |
| Integration test: hello world | ✔ | `just guest-run` — `examples/hello.rs` |

### Phase 2: Tool Dispatch

**Goal:** Host-registered tools callable from guest code via `call_tool()`.

| Task | Layer | Details |
|------|-------|---------|
| Add `tools.dispatch` to WIT | Guest | Import interface for host callbacks |
| Implement `ToolRegistry` in Rust | Rust | Register handlers, JSON dispatch, middleware hook (~10 lines for future capabilities) |
| Register `tools.dispatch` as host function | Rust | Bind ToolRegistry to WIT import on ProtoWasmSandbox |
| Guest-side `hyperlight` module | Guest | `call_tool()` wrapper calling `tools.dispatch()` |
| PyO3 `register_tool()` | Bindings | Python callable → Rust ToolRegistry |
| Integration test: tool round-trip | Test | Register tool in Python, call from guest, verify result |

**Exit criteria:** `sandbox.register_tool("add", lambda **kw: kw["a"] + kw["b"])` + guest `call_tool("add", a=1, b=2)` returns `3`.

### Phase 3: File I/O + Snapshots

**Goal:** WASI filesystem I/O, persistent file loading, and snapshot/restore.

| Task | Layer | Details |
|------|-------|---------|
| WASI filesystem preopens for `/input/`, `/output/` | Rust | Pre-populate /input/ from `inputs` dict, read /output/ after execution |
| `add_files()` / `add_file()` on sandbox | Rust + Bindings | Pre-load files that persist across runs. `add_files("a.json", "b.csv")` (varargs) reads from local FS; `add_file("name", bytes)` adds from memory |
| `inputs`/`outputs` params on `run()` | Bindings + SDK | Per-run overrides: `run(code, inputs={"data.json": b"..."}, outputs=["result.json"])` |
| Expose `snapshot()`/`restore()` | Bindings | Delegate to `LoadedWasmSandbox::snapshot()`/`restore()` |
| `SandboxEnvironment` dataclass | Python SDK | Configuration object |
| Integration tests for file I/O and snapshots | Test | Round-trip file data, verify snapshot rollback, verify persistent files |

**Exit criteria:** File I/O round-trips correctly. `add_files()` persists across runs. `snapshot()` → `run()` → `restore()` → `run()` produces independent results.

### Phase 3.5: WASI-HTTP Networking

**Goal:** Allow guest code to make outbound HTTP requests to explicitly allowlisted domains.

| Task | Layer | Details |
|------|-------|---------|
| Port WASI-HTTP host impl from http-example | Rust | Adapt [hyperlight-wasm-http-example](https://github.com/hyperlight-dev/hyperlight-wasm-http-example)'s `wasi:http/outgoing-handler@0.2.3` + supporting WASI interfaces (io, clocks, streams) |
| Add domain allowlist filtering | Rust | Wrap outgoing-handler to check `request.authority` against allowed domains |
| `add_network(domain)` method | Rust + Bindings | Adds domain to allowlist. Rejects all requests to non-listed domains |
| WASI-HTTP imports in guest WIT | Guest | Bundle full WASI interface chain (like http-example's hyperlight.wit) |
| Integration test: allowed + denied requests | Test | Verify allowed domain succeeds, non-allowed domain gets error |

**Exit criteria:** `sandbox.add_network("api.bing.com")` enables HTTP to that domain. Requests to other domains fail with a clear error.

### Phase 4: High-Level API + Capabilities Hook

**Goal:** Agent-friendly `CodeExecutionTool` and the middleware hook point.

| Task | Layer | Details |
|------|-------|---------|
| `CodeExecutionTool` class | Python SDK | Wraps WasmSandbox with environment config and tool list |
| `ToolRegistry.add_middleware()` | Rust | Pre-dispatch hook for future capabilities enforcement |
| Documentation and examples | Docs | README, usage examples, API reference |
| CI pipeline for guest component builds | CI | Automated componentize-py + AOT + packaging |

**Exit criteria:** `CodeExecutionTool` works end-to-end. Middleware hook exists and can be invoked (no enforcement logic yet).

---

## 10. Open Questions / Future Work

### Open Questions

1. ~~**Guest Python stdlib availability.**~~ **RESOLVED.** `io.StringIO`, `sys.stdout`/`stderr`, and `exec()` all work in componentize-py with `--stub-wasi`. Confirmed by Phase 1 end-to-end test.

2. **WASI filesystem mapping.** ~~Needs prototyping.~~ **DECIDED:** Use temp directory on host filesystem, mapped as WASI preopen. Requires removing `--stub-wasi` and implementing WASI host functions (Phase 3). The [hyperlight-wasm-http-example](https://github.com/hyperlight-dev/hyperlight-wasm-http-example) implements `wasi:io` and `wasi:cli` — we extend this pattern.

3. ~~**AOT format compatibility.**~~ **RESOLVED.** `hyperlight-wasm-aot compile --component` works with componentize-py output. The `--stub-wasi` flag is required for Phase 1 (no WASI host). Without it, loading fails with "incompatible object file format".

4. **Snapshot granularity.** Does `LoadedWasmSandbox::snapshot()` capture full Python interpreter state (heap, globals, etc.) or just Wasm linear memory? Need to verify that Python-level state is fully restored.

5. ~~**Guest module size.**~~ **RESOLVED.** AOT-compiled Python component is ~43MB. Too large for git. Must be CI-built artifact or separate download.

6. **Timeout implementation.** hyperlight-wasm's `max_execution_time` in `SandboxConfiguration` handles this at the hypervisor level. Verify it produces a clean `ExecutionResult` with appropriate exit code on timeout.

7. **WASI transition for Phase 3.** Phase 1 uses `--stub-wasi` which stubs out all WASI. Phase 3 needs real WASI for filesystem access. This means Phase 3 requires: (a) removing `--stub-wasi`, (b) implementing WASI host functions in Rust (port from http-example), (c) regenerating `host_bindgen!` from the WASI-enabled world. This is a significant step.

8. **WIT_WORLD env var.** hyperlight-wasm requires `WIT_WORLD` env var at runtime to resolve component export names. Need to figure out how to embed this into the PyO3 bindings so Python users don't need to set it manually.

### Future Work

- **Capabilities system (Phase 4+).** The middleware hook in `ToolRegistry` is the attachment point. Future work: define a capabilities schema, add `capabilities=` kwarg to `WasmSandbox`, enforce per-tool permissions.
- **`hyperlight-flex` unified SDK.** Separate package that provides a backend-agnostic API across hyperlight-unikraft and hyperlight_sandbox. Not this project's scope.
- **JavaScript guest.** componentize-py is Python-specific. A JS interpreter guest (via StarlingMonkey or similar) would be a separate guest component.
- **Streaming output.** Currently stdout/stderr returned after execution completes. Future: streaming callbacks during execution.
- **WASI networking (expanded).** Currently allowlisted by domain. Future: wildcard patterns, port-level control, rate limiting per domain.
- **Multi-sandbox pooling.** Pre-warm a pool of sandboxes for lower latency in high-throughput scenarios.
- **Custom guest builds.** Document and tooling for users to build custom guest components with additional Python packages bundled via componentize-py.

---

## Key References

| Resource | URL | Relevance |
|----------|-----|-----------|
| hyperlight-wasm (target repo) | `/home/jstur/projects/hyperlight-wasm-project/hyperlight-wasm` | Existing Wasm sandbox runtime we build on |
| hyperlight-unikraft (pattern reference) | `/home/jstur/projects/hyperlight-unikraft-project/hyperlight-unikraft` | `host/src/lib.rs` + `python/` — the API pattern to replicate |
| hyperlight-wasm-http-example | https://github.com/hyperlight-dev/hyperlight-wasm-http-example | **Critical reference.** Proves WASI HTTP works with hyperlight-wasm. Full implementation of `wasi:http@0.2.3`, `wasi:io@0.2.3`, `wasi:clocks@0.2.3`, `wasi:random@0.2.3`, `wasi:cli@0.2.3` as hyperlight host functions. Blueprint for our WASI filesystem and HTTP implementations. |
| componentize-py sandbox example | https://github.com/bytecodealliance/componentize-py/tree/main/examples/sandbox | Pattern for compiling Python interpreter into Wasm component |
| hyperlight-flex | https://github.com/danbugs/hyperlight-flex | Future unified SDK that shims unikraft and wasm backends (not this project's scope) |
