"""Basic example of hyperlight_sandbox Python SDK."""

from hyperlight_sandbox import WasmSandbox

# Create sandbox pointing to the AOT-compiled Python component
sandbox = WasmSandbox(module_path="src/python_sandbox/python-sandbox.aot")

# Register host tools before first run()
sandbox.register_tool("add", lambda a=0, b=0: a + b)
sandbox.register_tool("greet", lambda name="world": f"Hello, {name}!")

# Test 1: Basic code execution
print("--- Test 1: Basic execution ---")
result = sandbox.run('print("hello from python SDK!")')
print(f"stdout: {result.stdout!r}")
print(f"success: {result.success}")

# Test 2: Tool dispatch via call_tool()
print("\n--- Test 2: Tool dispatch ---")
result = sandbox.run("""
result = call_tool('add', a=3, b=4)
greeting = call_tool('greet', name='James')
print(f"3 + 4 = {result}")
print(f"{greeting}")
try:
    call_tool('nonexistent', x=1)
except RuntimeError as e:
    print(f"Caught error: {e}")
print("All tool tests passed!")
""")
print(f"stdout: {result.stdout!r}")
print(f"success: {result.success}")

# Test 3: Sandbox reuse
print("\n--- Test 3: Sandbox reuse ---")
result = sandbox.run('print("second run works!")')
print(f"stdout: {result.stdout!r}")
print(f"success: {result.success}")

# Test 4: Snapshot/restore
print("\n--- Test 4: Snapshot/restore ---")
snap = sandbox.snapshot()
result1 = sandbox.run('x = 42; print(f"x = {x}")')
print(f"Before restore: {result1.stdout!r}")
sandbox.restore(snap)
result2 = sandbox.run("""
try:
    print(f"x = {x}")
except NameError:
    print("x is not defined (state was rolled back)")
""")
print(f"After restore: {result2.stdout!r}")

print("\n✅ All tests passed!")
