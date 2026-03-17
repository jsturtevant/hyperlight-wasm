"""Basic example of hyperlight_sandbox Python SDK."""

import time
from hyperlight_sandbox import WasmSandbox

def timed_run(sandbox, code, label="run"):
    """Run code in sandbox and print timing."""
    start = time.perf_counter()
    result = sandbox.run(code)
    elapsed_ms = (time.perf_counter() - start) * 1000
    print(f"⏱️  {label}: {elapsed_ms:.1f}ms")
    return result

# Create sandbox pointing to the AOT-compiled Python component
t0 = time.perf_counter()
sandbox = WasmSandbox(module_path="src/python_sandbox/python-sandbox.aot")
print(f"⏱️  WasmSandbox created (lazy): {(time.perf_counter() - t0) * 1000:.1f}ms")

# Register host tools before first run()
sandbox.register_tool("add", lambda a=0, b=0: a + b)
sandbox.register_tool("greet", lambda name="world": f"Hello, {name}!")

# Test 1: Basic code execution (first run triggers sandbox init)
print("\n--- Test 1: Basic execution (includes sandbox init) ---")
result = timed_run(sandbox, 'print("hello from python SDK!")', "first run (cold)")
print(f"stdout: {result.stdout!r}")
print(f"success: {result.success}")

# Test 2: Tool dispatch via call_tool()
print("\n--- Test 2: Tool dispatch ---")
result = timed_run(sandbox, """
result = call_tool('add', a=3, b=4)
greeting = call_tool('greet', name='James')
print(f"3 + 4 = {result}")
print(f"{greeting}")
try:
    call_tool('nonexistent', x=1)
except RuntimeError as e:
    print(f"Caught error: {e}")
print("All tool tests passed!")
""", "tool dispatch")
print(f"stdout: {result.stdout!r}")
print(f"success: {result.success}")

# Test 3: Sandbox reuse
print("\n--- Test 3: Sandbox reuse ---")
result = timed_run(sandbox, 'print("second run works!")', "reuse (warm)")
print(f"stdout: {result.stdout!r}")
print(f"success: {result.success}")

# Test 4: Snapshot/restore
print("\n--- Test 4: Snapshot/restore ---")
t0 = time.perf_counter()
snap = sandbox.snapshot()
print(f"⏱️  snapshot: {(time.perf_counter() - t0) * 1000:.1f}ms")

result1 = timed_run(sandbox, 'x = 42; print(f"x = {x}")', "pre-restore run")
print(f"Before restore: {result1.stdout!r}")

t0 = time.perf_counter()
sandbox.restore(snap)
print(f"⏱️  restore: {(time.perf_counter() - t0) * 1000:.1f}ms")

result2 = timed_run(sandbox, """
try:
    print(f"x = {x}")
except NameError:
    print("x is not defined (state was rolled back)")
""", "post-restore run")
print(f"After restore: {result2.stdout!r}")

print("\n✅ All tests passed!")
