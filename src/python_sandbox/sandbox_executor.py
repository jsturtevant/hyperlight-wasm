"""Guest-side executor that runs inside the Wasm component.

Phase 1: Basic code execution with stdout/stderr capture.
No tools import yet — call_tool raises NotImplementedError.

componentize-py expects a class `Executor` matching the WIT `executor` interface,
with a `run` method returning an `ExecutionResult` dataclass.
"""

import sys
import io

from wit_world.exports.executor import ExecutionResult


def _call_tool(name: str, **kwargs):
    """Stub for Phase 1. Phase 2 will route through WIT tools.dispatch."""
    raise NotImplementedError(
        f"call_tool('{name}') is not available in Phase 1. "
        "Tool dispatch requires Phase 2 WIT (tools.dispatch import)."
    )


class Executor:
    """Implements the WIT executor interface for componentize-py."""

    def run(self, code: str) -> ExecutionResult:
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

        return ExecutionResult(
            stdout=stdout_capture.getvalue(),
            stderr=stderr_capture.getvalue(),
            exit_code=exit_code,
        )
