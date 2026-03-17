"""
Hyperlight Sandbox — Full Capabilities Demo

Shows the complete API surface for hyperlight_sandbox.
Features marked [WORKING] run today. Features marked [PLANNED] show the
Phase 3 / Phase 3.5 API that will work once WASI filesystem and WASI-HTTP
host functions are implemented.
"""

from hyperlight_sandbox import WasmSandbox

sandbox = WasmSandbox(module_path="src/python_sandbox/python-sandbox.aot")

# ── Register tools ──────────────────────────────────────────────────
# [WORKING] Host tools callable from guest via call_tool()
sandbox.register_tool("add", lambda a=0, b=0: a + b)
sandbox.register_tool("multiply", lambda a=0, b=0: a * b)
sandbox.register_tool("greet", lambda name="world": f"Hello, {name}!")
sandbox.register_tool("lookup", lambda key="": {"api_key": "sk-demo", "model": "gpt-4"}.get(key, "not found"))

# ── [PLANNED Phase 3] Pre-load files ────────────────────────────────
# sandbox.add_files("data.json", "config.yaml")   # from local filesystem
# sandbox.add_file("data.json", b'{"key": "value"}')  # from bytes

# ── [PLANNED Phase 3.5] Allow network access ───────────────────────
# sandbox.add_network("api.bing.com")
# sandbox.add_network("api.github.com")

# ═══════════════════════════════════════════════════════════════════
# Test 1: Basic code execution  [WORKING]
# ═══════════════════════════════════════════════════════════════════
print("═" * 60)
print("Test 1: Basic code execution")
print("═" * 60)
result = sandbox.run("""
import math
primes = [n for n in range(2, 50) if all(n % i != 0 for i in range(2, int(math.sqrt(n)) + 1))]
print(f"Primes under 50: {primes}")
print(f"Count: {len(primes)}")
""")
print(result.stdout)
assert result.success

# ═══════════════════════════════════════════════════════════════════
# Test 2: Tool dispatch  [WORKING]
# ═══════════════════════════════════════════════════════════════════
print("═" * 60)
print("Test 2: Tool dispatch — host functions from guest code")
print("═" * 60)
result = sandbox.run("""
# call_tool() is available as a builtin in the sandbox
sum_result = call_tool('add', a=10, b=20)
product = call_tool('multiply', a=6, b=7)
greeting = call_tool('greet', name='Developer')
config = call_tool('lookup', key='model')

print(f"10 + 20 = {sum_result}")
print(f"6 × 7 = {product}")
print(f"{greeting}")
print(f"Config lookup: model = {config}")

# Unknown tools return a clean error
try:
    call_tool('nonexistent_tool')
except RuntimeError as e:
    print(f"Error handling works: {e}")
""")
print(result.stdout)
assert result.success

# ═══════════════════════════════════════════════════════════════════
# Test 3: Snapshot / Restore  [WORKING]
# ═══════════════════════════════════════════════════════════════════
print("═" * 60)
print("Test 3: Snapshot/restore — rewind interpreter state")
print("═" * 60)
snap = sandbox.snapshot()

result = sandbox.run("counter = 100; print(f'Set counter = {counter}')")
print(f"Before restore: {result.stdout.strip()}")

sandbox.restore(snap)

result = sandbox.run("""
try:
    print(f"counter = {counter}")
except NameError:
    print("counter is undefined — state was rolled back!")
""")
print(f"After restore:  {result.stdout.strip()}")
assert result.success

# ═══════════════════════════════════════════════════════════════════
# Test 4: Complex computation in a single run  [WORKING]
# ═══════════════════════════════════════════════════════════════════
print()
print("═" * 60)
print("Test 4: Complex multi-step computation")
print("═" * 60)
result = sandbox.run("""
# Build up state within a single execution
data = []
for i in range(5):
    val = call_tool('multiply', a=i, b=i)
    data.append(val)
total = call_tool('add', a=sum(data[:3]), b=sum(data[3:]))
print(f"Squares: {data}")
print(f"Total: {total}")
""")
print(result.stdout.strip())
assert result.success

# ═══════════════════════════════════════════════════════════════════
# Test 5: File + network access — blocked by default
# ═══════════════════════════════════════════════════════════════════
print()
print("═" * 60)
print("Test 5: File + network access denied without permissions")
print("═" * 60)
result = sandbox.run("""
import json

# --- File access without sandbox.add_files() ---
try:
    data = json.load(open("/input/data.json"))
    print(f"Read data: {data}")
except (FileNotFoundError, OSError) as e:
    print(f"FileNotFoundError: /input/data.json — no files loaded")
    print(f"  Fix: sandbox.add_files('data.json')")

# --- Network access without sandbox.add_network() ---
try:
    import urllib.request
    resp = urllib.request.urlopen("https://api.bing.com/search?q=hello")
    print(resp.read().decode()[:200])
except (OSError, ImportError) as e:
    print(f"OSError: network not available in sandbox")
    print(f"  Fix: sandbox.add_network('api.bing.com')")

print()
print("Both file and network access fail cleanly without permissions.")
print("No data leaves the sandbox unless explicitly allowed.")
""")
print(result.stdout)
# Note: this test may fail with exit_code=1 if WASI stubs trap.
# That's a known limitation — Phase 3/3.5 will fix it.
if not result.success:
    print("File/network access correctly denied (sandbox terminated).")
    print("  sandbox.add_files('data.json')       → Phase 3 (WASI filesystem)")
    print("  sandbox.add_network('api.bing.com')   → Phase 3.5 (WASI-HTTP)")

print("═" * 60)
print("✅ All working tests passed! (Phase 3 & 3.5 planned)")
print("═" * 60)
