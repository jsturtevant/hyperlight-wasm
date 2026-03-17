# hyperlight-sandbox

Python SDK for running untrusted code inside hardware-isolated WebAssembly sandboxes powered by [hyperlight-wasm](https://github.com/hyperlight-dev/hyperlight-wasm).

## Quick Start

```python
from hyperlight_sandbox import WasmSandbox

sandbox = WasmSandbox(module_path="python-sandbox.aot")
sandbox.register_tool("add", lambda a=0, b=0: a + b)

result = sandbox.run('''
result = call_tool('add', a=3, b=4)
print(result)
''')
print(result.stdout)  # "7\n"
```

## Build

```bash
cd python
maturin develop
```
