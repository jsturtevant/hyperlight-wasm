//! High-level host library for running Python code in hyperlight-wasm sandboxes.
//!
//! This crate wraps the hyperlight-wasm `SandboxBuilder` chain into a
//! convenient [`PythonSandbox`] that builds, loads, and executes guest code
//! in a single API surface. It also provides a [`ToolRegistry`] stub that
//! will be wired to WIT host-function imports in Phase 2.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use hyperlight_wasm::{LoadedWasmSandbox, SandboxBuilder, Snapshot};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for building a [`PythonSandbox`].
pub struct SandboxConfig {
    /// Path to the AOT-compiled Wasm component (e.g. `python-sandbox.aot`).
    pub module_path: String,
    /// Guest heap size in bytes.
    pub heap_size: u64,
    /// Guest scratch / stack size in bytes.
    pub stack_size: u64,
    /// Optional wall-clock timeout (seconds) for guest execution.
    pub timeout_secs: Option<u64>,
}

// ---------------------------------------------------------------------------
// Execution result
// ---------------------------------------------------------------------------

/// The result of executing code inside the sandbox.
///
/// Mirrors the WIT `execution-result` record:
/// ```wit
/// record execution-result {
///     stdout: string,
///     stderr: string,
///     exit-code: s32,
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

// ---------------------------------------------------------------------------
// Tool registry (Phase 1 stub — functional but not yet wired to host fns)
// ---------------------------------------------------------------------------

/// Registry of host-side tool handlers.
///
/// In Phase 2 the registry will be connected to the `tools.dispatch` WIT
/// import so that guest code can call registered tools via `call_tool()`.
/// For now the registry is fully functional (register / dispatch / middleware)
/// but is not yet bound to a host function on the sandbox.
pub struct ToolRegistry {
    tools: HashMap<
        String,
        Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value> + Send + Sync>,
    >,
    /// Middleware hook point for future capabilities enforcement.
    /// Phase 4 will populate this with pre-dispatch checks.
    middleware: Vec<Box<dyn Fn(&str, &serde_json::Value) -> Result<()> + Send + Sync>>,
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            middleware: Vec::new(),
        }
    }

    /// Register a tool handler under `name`.
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
        for hook in &self.middleware {
            hook(name, &args)?;
        }

        let handler = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown tool: {}", name))?;

        handler(args)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// PythonSandbox
// ---------------------------------------------------------------------------

/// A ready-to-use Python execution sandbox backed by hyperlight-wasm.
///
/// Wraps the full hyperlight-wasm lifecycle:
/// `SandboxBuilder` → `ProtoWasmSandbox` → `WasmSandbox` → `LoadedWasmSandbox`
pub struct PythonSandbox {
    loaded: LoadedWasmSandbox,
    tools: ToolRegistry,
}

impl PythonSandbox {
    /// Build a new sandbox from the given configuration.
    ///
    /// This walks the hyperlight-wasm chain:
    /// 1. `SandboxBuilder::new()` with heap/scratch sizes
    /// 2. `.build()` → `ProtoWasmSandbox`
    /// 3. `.load_runtime()` → `WasmSandbox`
    /// 4. `.load_module(module_path)` → `LoadedWasmSandbox`
    pub fn new(config: SandboxConfig) -> Result<Self> {
        let module_path = config.module_path.clone();

        let sandbox = SandboxBuilder::new()
            .with_guest_heap_size(config.heap_size)
            .with_guest_scratch_size(config.stack_size as usize)
            .build()
            .context("failed to build ProtoWasmSandbox")?;

        // Phase 2 will register the tools.dispatch host function here,
        // between build() and load_runtime().

        let wasm_sandbox = sandbox
            .load_runtime()
            .context("failed to load Wasm runtime")?;

        let loaded = wasm_sandbox
            .load_module(Path::new(&module_path))
            .context("failed to load Wasm module")?;

        Ok(Self {
            loaded,
            tools: ToolRegistry::new(),
        })
    }

    /// Execute Python `code` in the sandbox.
    ///
    /// Calls the guest's `executor.run(code)` function. The guest returns a
    /// JSON-serialised `execution-result` record which is deserialised into
    /// an [`ExecutionResult`].
    pub fn run(&mut self, code: &str) -> Result<ExecutionResult> {
        let raw: String = self
            .loaded
            .call_guest_function("run", code.to_string())
            .map_err(|e| anyhow::anyhow!("guest call failed: {e}"))?;

        let result: ExecutionResult =
            serde_json::from_str(&raw).context("failed to deserialise execution-result")?;

        Ok(result)
    }

    /// Take a snapshot of the current sandbox state (Wasm linear memory).
    pub fn snapshot(&mut self) -> Result<Arc<Snapshot>> {
        self.loaded
            .snapshot()
            .map_err(|e| anyhow::anyhow!("snapshot failed: {e}"))
    }

    /// Restore the sandbox to a previously captured snapshot.
    pub fn restore(&mut self, snapshot: Arc<Snapshot>) -> Result<()> {
        self.loaded
            .restore(snapshot)
            .map_err(|e| anyhow::anyhow!("restore failed: {e}"))
    }

    /// Access the tool registry (e.g. to register tools before Phase 2 wiring).
    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    /// Mutably access the tool registry.
    pub fn tools_mut(&mut self) -> &mut ToolRegistry {
        &mut self.tools
    }
}
