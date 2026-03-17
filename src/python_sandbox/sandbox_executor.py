"""Guest-side executor that runs inside the Wasm component.

Phase 2: Code execution with tool dispatch via WIT tools.dispatch import.
Guest code can call host tools via call_tool('name', key=val).

componentize-py expects a class `Executor` matching the WIT `executor` interface,
with a `run` method returning an `ExecutionResult` dataclass.
"""

import sys
import io
import json

from wit_world.exports.executor import ExecutionResult
import wit_world.imports.tools as tools


def _call_tool(tool_name: str, **kwargs):
    """Call a host-registered tool via WIT tools.dispatch."""
    request = json.dumps({"name": tool_name, "args": kwargs})
    result_json = tools.dispatch(request)
    result = json.loads(result_json)
    if "error" in result:
        raise RuntimeError(f"Tool '{tool_name}' failed: {result['error']}")
    return result.get("result")


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
