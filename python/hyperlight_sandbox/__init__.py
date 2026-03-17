"""
hyperlight_sandbox — Python SDK for Wasm-isolated code execution

Execute untrusted Python code inside hardware-isolated WebAssembly sandboxes
powered by hyperlight-wasm. Designed for LLM agent frameworks.

Usage:
    from hyperlight_sandbox import WasmSandbox

    sandbox = WasmSandbox(module_path="python-sandbox.aot")
    sandbox.register_tool("add", lambda a=0, b=0: a + b)

    result = sandbox.run('''
    result = call_tool('add', a=3, b=4)
    print(result)
    ''')
    print(result.stdout)  # "7\n"

High-level usage:
    from hyperlight_sandbox import CodeExecutionTool, SandboxEnvironment

    tool = CodeExecutionTool(
        environment=SandboxEnvironment(module_path="python-sandbox.aot"),
        tools=[my_tool_fn],
        timeout=30,
    )
    result = tool.run(code="print(1+1)")
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from typing import Any, Callable

# Import native module (built by maturin)
try:
    from hyperlight_sandbox._native import WasmSandbox, PyExecutionResult, PySnapshot, __version__
except ImportError as e:
    raise ImportError(
        "Native module not found. Build with: cd python && maturin develop"
    ) from e


__all__ = [
    "WasmSandbox",
    "SandboxEnvironment",
    "CodeExecutionTool",
    "ExecutionResult",
    "__version__",
]


@dataclass
class SandboxEnvironment:
    """Configuration for the Wasm sandbox environment."""

    module_path: str = "python-sandbox.aot"
    heap_size: str = "200Mi"
    stack_size: str = "100Mi"


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
            for tool_fn in self.tools:
                self._sandbox.register_tool(tool_fn)
        return self._sandbox

    def run(
        self,
        code: str,
        inputs: dict[str, bytes] | None = None,
        outputs: list[str] | None = None,
    ) -> ExecutionResult:
        """Execute code in the persistent sandbox.

        Args:
            code: Python source code to execute inside the sandbox
            inputs: Files to make available at /input/ (future, Phase 3)
            outputs: Output files to retrieve from /output/ (future, Phase 3)

        Returns:
            ExecutionResult with stdout, stderr, and exit_code
        """
        sandbox = self._get_sandbox()
        native_result = sandbox.run(code)

        return ExecutionResult(
            stdout=native_result.stdout,
            stderr=native_result.stderr,
            exit_code=native_result.exit_code,
        )
