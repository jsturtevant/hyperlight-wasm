use std::collections::HashMap;

use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

use hyperlight_sandbox::{PythonSandbox, SandboxConfig, Snapshot, ToolRegistry};

/// Parse a memory size string like "512Mi" or "8Mi" to bytes.
fn parse_size(s: &str) -> PyResult<u64> {
    let s = s.trim();
    if let Some(n) = s.strip_suffix("Gi") {
        Ok(n.parse::<u64>()
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid size: {e}")))?
            * 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("Mi") {
        Ok(n.parse::<u64>()
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid size: {e}")))?
            * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("Ki") {
        Ok(n.parse::<u64>()
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid size: {e}")))?
            * 1024)
    } else {
        s.parse::<u64>()
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid size: {e}")))
    }
}

#[pyclass(unsendable)]
pub struct WasmSandbox {
    inner: Option<PythonSandbox>,
    tools: HashMap<String, Py<PyAny>>,
    pending_files: Vec<(String, Vec<u8>)>,
    pending_networks: Vec<(String, Option<Vec<String>>)>,
    module_path: String,
    heap_size: u64,
    stack_size: u64,
    timeout_secs: Option<u64>,
}

#[pymethods]
impl WasmSandbox {
    #[new]
    #[pyo3(signature = (module_path="python-sandbox.aot", heap_size="200Mi", stack_size="100Mi", timeout_secs=None))]
    fn new(
        module_path: &str,
        heap_size: &str,
        stack_size: &str,
        timeout_secs: Option<u64>,
    ) -> PyResult<Self> {
        Ok(WasmSandbox {
            inner: None,
            tools: HashMap::new(),
            pending_files: Vec::new(),
            pending_networks: Vec::new(),
            module_path: module_path.to_string(),
            heap_size: parse_size(heap_size)?,
            stack_size: parse_size(stack_size)?,
            timeout_secs,
        })
    }

    #[pyo3(signature = (name_or_tool, callback=None))]
    fn register_tool(
        &mut self,
        py: Python<'_>,
        name_or_tool: Py<PyAny>,
        callback: Option<Py<PyAny>>,
    ) -> PyResult<()> {
        if self.inner.is_some() {
            return Err(PyRuntimeError::new_err(
                "Cannot register tools after sandbox has been initialized. \
                 Register all tools before the first run() call.",
            ));
        }
        let obj = name_or_tool.bind(py);
        let (name, cb) = if callback.is_none()
            && obj.hasattr("handler")?
            && obj.hasattr("name")?
        {
            let name: String = obj.getattr("name")?.extract()?;
            let wrapper = make_sdk_tool_wrapper(py, obj)?;
            (name, wrapper)
        } else {
            let name: String = obj.extract()?;
            let cb = callback.ok_or_else(|| {
                PyTypeError::new_err(
                    "register_tool() expects (name, callable) or a Tool object",
                )
            })?;
            (name, cb)
        };
        self.tools.insert(name, cb);
        Ok(())
    }

    #[pyo3(signature = (code))]
    fn run(&mut self, py: Python<'_>, code: &str) -> PyResult<PyExecutionResult> {
        if self.inner.is_none() {
            self.initialize_sandbox(py)?;
        }
        let sandbox = self.inner.as_mut().expect("sandbox initialized");
        let result = sandbox
            .run(code)
            .map_err(|e| PyRuntimeError::new_err(format!("Execution failed: {e}")))?;
        let outputs = PyDict::new(py);
        for (name, data) in &result.outputs {
            outputs.set_item(name, pyo3::types::PyBytes::new(py, data))?;
        }
        Ok(PyExecutionResult {
            stdout: result.stdout,
            stderr: result.stderr,
            exit_code: result.exit_code,
            outputs: outputs.unbind(),
        })
    }

    #[pyo3(signature = (target, methods=None))]
    fn allow(&mut self, target: &str, methods: Option<Vec<String>>) -> PyResult<()> {
        if let Some(sandbox) = self.inner.as_mut() {
            sandbox.allow(target, methods);
        } else {
            self.pending_networks.push((target.to_string(), methods));
        }
        Ok(())
    }

    /// Backward-compatible alias for `allow(domain)`.
    fn add_network(&mut self, domain: &str) -> PyResult<()> {
        self.allow(domain, None)
    }

    fn add_file(&mut self, name: &str, data: &[u8]) -> PyResult<()> {
        if let Some(sandbox) = self.inner.as_mut() {
            sandbox.add_file(name, data.to_vec());
        } else {
            self.pending_files.push((name.to_string(), data.to_vec()));
        }
        Ok(())
    }

    #[pyo3(signature = (*paths))]
    fn add_files(&mut self, paths: Vec<String>) -> PyResult<()> {
        for path in &paths {
            let p = std::path::Path::new(path);
            let name = p.file_name()
                .ok_or_else(|| PyRuntimeError::new_err(format!("Invalid path: {path}")))?;
            let data = std::fs::read(p)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to read {path}: {e}")))?;
            let name = name.to_string_lossy().to_string();
            if let Some(sandbox) = self.inner.as_mut() {
                sandbox.add_file(&name, data);
            } else {
                self.pending_files.push((name, data));
            }
        }
        Ok(())
    }

    fn snapshot(&mut self) -> PyResult<PySnapshot> {
        let sandbox = self.inner.as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Sandbox not initialized"))?;
        let snap = sandbox.snapshot()
            .map_err(|e| PyRuntimeError::new_err(format!("Snapshot failed: {e}")))?;
        Ok(PySnapshot { inner: snap })
    }

    fn restore(&mut self, snapshot: &PySnapshot) -> PyResult<()> {
        let sandbox = self.inner.as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Sandbox not initialized"))?;
        sandbox.restore(&snapshot.inner)
            .map_err(|e| PyRuntimeError::new_err(format!("Restore failed: {e}")))?;
        Ok(())
    }
}

impl WasmSandbox {
    fn initialize_sandbox(&mut self, py: Python<'_>) -> PyResult<()> {
        let mut registry = ToolRegistry::new();
        let tools = std::mem::take(&mut self.tools);
        for (name, callback) in tools {
            let cb = callback.clone_ref(py);
            registry.register(&name, move |args: serde_json::Value| {
                Python::attach(|py| {
                    let kwargs = PyDict::new(py);
                    if let serde_json::Value::Object(map) = &args {
                        for (k, v) in map {
                            let py_val = json_to_py(py, v)?;
                            kwargs.set_item(k, py_val)?;
                        }
                    }
                    let result = cb.call(py, (), Some(&kwargs))?;
                    py_to_json(result.bind(py))
                })
                .map_err(|e: PyErr| anyhow::anyhow!("{e}"))
            });
        }
        let config = SandboxConfig {
            module_path: self.module_path.clone(),
            heap_size: self.heap_size,
            stack_size: self.stack_size,
            timeout_secs: self.timeout_secs,
        };
        let mut sandbox = PythonSandbox::with_tools(config, registry)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create sandbox: {e}")))?;
        // Load any files queued before sandbox init
        let pending = std::mem::take(&mut self.pending_files);
        for (name, data) in pending {
            sandbox.add_file(&name, data);
        }
        // Apply any networks queued before sandbox init
        let pending_networks = std::mem::take(&mut self.pending_networks);
        for (target, methods) in pending_networks {
            sandbox.allow(&target, methods);
        }
        self.inner = Some(sandbox);
        Ok(())
    }
}

/// Create a Python wrapper around an SDK Tool object that handles async handlers.
///
/// SDK `define_tool()` wraps handlers as async coroutines. The sandbox dispatches
/// tools synchronously, so we need a sync wrapper that properly awaits the async handler
/// and unwraps the SDK's `textResultForLlm` envelope.
fn make_sdk_tool_wrapper(py: Python<'_>, tool: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let wrapper_module = PyModule::from_code(
        py,
        c"
def _make_sdk_wrapper(tool):
    import ast
    import asyncio
    import concurrent.futures

    def _invoke_sync(invocation):
        return asyncio.run(tool.handler(invocation))

    def wrapper(**kwargs):
        invocation = {
            'arguments': kwargs,
            'tool_call_id': 'sandbox',
            'tool_name': tool.name,
        }

        try:
            loop = asyncio.get_running_loop()
        except RuntimeError:
            loop = None

        if loop and loop.is_running():
            with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
                result = pool.submit(_invoke_sync, invocation).result()
        else:
            result = _invoke_sync(invocation)

        # The SDK serializes return values to textResultForLlm (a string).
        # Deserialize back to a Python object so guest code gets real types.
        if isinstance(result, dict) and 'textResultForLlm' in result:
            text = result['textResultForLlm']
            try:
                return ast.literal_eval(text)
            except (ValueError, SyntaxError):
                return text
        return result

    return wrapper
",
        c"_sdk_tool_wrapper.py",
        c"_sdk_tool_wrapper",
    )?;

    let make_wrapper = wrapper_module.getattr("_make_sdk_wrapper")?;
    Ok(make_wrapper.call1((tool,))?.unbind())
}

fn json_to_py(py: Python<'_>, val: &serde_json::Value) -> PyResult<Py<PyAny>> {
    match val {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok((*b).into_pyobject(py)?.to_owned().into_any().unbind()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any().unbind())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.into_any().unbind())
            } else {
                Ok(py.None())
            }
        }
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        serde_json::Value::Array(arr) => {
            let list = pyo3::types::PyList::empty(py);
            for item in arr {
                list.append(json_to_py(py, item)?)?;
            }
            Ok(list.into_any().unbind())
        }
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(dict.into_any().unbind())
        }
    }
}

fn py_to_json(obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if obj.is_none() {
        Ok(serde_json::Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(serde_json::Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(serde_json::json!(i))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(serde_json::json!(f))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(serde_json::Value::String(s))
    } else if let Ok(list) = obj.downcast::<pyo3::types::PyList>() {
        let arr: PyResult<Vec<serde_json::Value>> =
            list.iter().map(|item| py_to_json(&item)).collect();
        Ok(serde_json::Value::Array(arr?))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            map.insert(key, py_to_json(&v)?);
        }
        Ok(serde_json::Value::Object(map))
    } else {
        let s = obj.str()?.to_string();
        Ok(serde_json::Value::String(s))
    }
}

#[pyclass]
pub struct PyExecutionResult {
    #[pyo3(get)]
    pub stdout: String,
    #[pyo3(get)]
    pub stderr: String,
    #[pyo3(get)]
    pub exit_code: i32,
    #[pyo3(get)]
    pub outputs: Py<PyDict>,
}

#[pymethods]
impl PyExecutionResult {
    #[getter]
    fn success(&self) -> bool {
        self.exit_code == 0
    }

    fn __repr__(&self) -> String {
        format!(
            "ExecutionResult(exit_code={}, stdout={:?}, stderr={:?})",
            self.exit_code, self.stdout, self.stderr,
        )
    }
}

#[pyclass]
pub struct PySnapshot {
    inner: std::sync::Arc<Snapshot>,
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<WasmSandbox>()?;
    m.add_class::<PyExecutionResult>()?;
    m.add_class::<PySnapshot>()?;
    m.add("__version__", "0.1.0")?;
    Ok(())
}
