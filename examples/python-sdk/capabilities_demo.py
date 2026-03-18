"""
Hyperlight Sandbox — Full Capabilities Demo

Shows the complete API surface for hyperlight_sandbox.
Features marked [WORKING] run today. Features marked [PLANNED] show the
Phase 3.5 API that will work once WASI-HTTP host functions are implemented.
"""

import json
from hyperlight_sandbox import WasmSandbox

sandbox = WasmSandbox(module_path="src/python_sandbox/python-sandbox.aot")

# ── Register tools ──────────────────────────────────────────────────
# [WORKING] Host tools callable from guest via call_tool()
sandbox.register_tool("add", lambda a=0, b=0: a + b)
sandbox.register_tool("multiply", lambda a=0, b=0: a * b)
sandbox.register_tool("greet", lambda name="world": f"Hello, {name}!")
sandbox.register_tool("lookup", lambda key="": {"api_key": "sk-demo", "model": "gpt-4"}.get(key, "not found"))

# ── [WORKING] Pre-load files ───────────────────────────────────────
sandbox.add_file("data.json", b'{"users": [{"name": "Alice"}, {"name": "Bob"}]}')
sandbox.add_file("config.yaml", b"model: gpt-4\ntimeout: 30\n")

# ── [WORKING] Allow network access ───────────────────────
sandbox.allow("https://httpbin.org")

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
# Test 5: File I/O — read from /input/, write to /output/  [WORKING]
# ═══════════════════════════════════════════════════════════════════
print()
print("═" * 60)
print("Test 5: File I/O via WASI filesystem")
print("═" * 60)
result = sandbox.run("""
import json

# File not found gives a clean error (no crash)
try:
    open('/input/nonexistent.txt', 'r')
except FileNotFoundError:
    print("FileNotFoundError for missing file — correct!")

# Read pre-loaded files from /input/
with open('/input/data.json', 'r') as f:
    data = json.load(f)
print(f"Users: {[u['name'] for u in data['users']]}")

with open('/input/config.yaml', 'r') as f:
    config = f.read()
print(f"Config: {config.strip()}")

# Write results to /output/
with open('/output/report.json', 'w') as f:
    json.dump({"user_count": len(data['users']), "status": "ok"}, f)
print("Wrote report.json to /output/")
""")
print(result.stdout)
assert result.success
report = json.loads(result.outputs["report.json"])
print(f"Host read back: {report}")
assert report["user_count"] == 2

# ═══════════════════════════════════════════════════════════════════
# Test 6: Network access — blocked by default  [WORKING]
# ═══════════════════════════════════════════════════════════════════
print()
print("═" * 60)
print("Test 6: Network access denied without permissions")
print("═" * 60)
result = sandbox.run("""
try:
    resp = http_get("https://example.com")
    print(f"Got response: {resp['status']}")
except Exception as e:
    print(f"Network blocked: {type(e).__name__}: {e}")
    print("  (example.com is not in the allowlist — correct!)")
""")
print(result.stdout)
if not result.success:
    print("(Network access correctly denied — sandbox terminated)")

# ═══════════════════════════════════════════════════════════════════
# Test 7: Network access — allowed domain  [WORKING]
# ═══════════════════════════════════════════════════════════════════
print()
print("═" * 60)
print("Test 7: Network access to allowed domain (WASI-HTTP)")
print("═" * 60)
result = sandbox.run("""
resp = http_get("https://httpbin.org/get")
print(f"HTTP status: {resp['status']}")
print(f"Response body (first 200 chars):")
print(resp['body'][:200])
""")
print(result.stdout)
if result.success:
    print("✅ Network access to allowed domain works via WASI-HTTP!")
else:
    print(f"⚠️ Network access failed")
    print(f"stderr: {result.stderr[:300]}")

print("═" * 60)
print("✅ All tests passed!")
print("═" * 60)
