"""Guest-side executor that runs inside the Wasm component.

Phase 1: Basic code execution with stdout/stderr capture.
No tools import yet — call_tool raises NotImplementedError.
"""

import sys
import io


def _call_tool(name: str, **kwargs):
    """Stub for Phase 1. Phase 2 will route through WIT tools.dispatch."""
    raise NotImplementedError(
        f"call_tool('{name}') is not available in Phase 1. "
        "Tool dispatch requires Phase 2 WIT (tools.dispatch import)."
    )


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
