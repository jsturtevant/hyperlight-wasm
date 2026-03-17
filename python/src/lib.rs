use std::collections::HashMap;

use pyo3::exceptions::PyRuntimeError;
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
            let handler: Py<PyAny> = obj.getattr("handler")?.unbind();
            (name, handler)
        } else {
            let name: String = obj.extract()?;
            let cb = callback.ok_or_else(|| {
                PyRuntimeError::new_err(
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
        Ok(PyExecutionResult {
            stdout: result.stdout,
            stderr: result.stderr,
            exit_code: result.exit_code,
        })
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
        let sandbox = PythonSandbox::with_tools(config, registry)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create sandbox: {e}")))?;
        self.inner = Some(sandbox);
        Ok(())
    }
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
#[derive(Clone)]
pub struct PyExecutionResult {
    #[pyo3(get)]
    pub stdout: String,
    #[pyo3(get)]
    pub stderr: String,
    #[pyo3(get)]
    pub exit_code: i32,
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
