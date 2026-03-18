# python/ Cargo.toml: missing [patch.crates-io] for hyperlight-component-util

**Repo**: `jsturtevant/hyperlight-wasm` (this repo, not upstream)  
**Severity**: Build hangs or uses wrong crate version  
**Workaround**: Add the patch section

## Problem

The `python/Cargo.toml` workspace is standalone (not part of the main hyperlight-wasm workspace). It has a `[patch."https://github.com/hyperlight-dev/hyperlight"]` section to use the local vendored `hyperlight-component-util`, but is missing `[patch.crates-io]`.

This causes `maturin develop` to either:
- Hang indefinitely trying to resolve the crate from crates.io
- Use the wrong version of `hyperlight-component-util` (without local fixes)

## Fix

Add to `python/Cargo.toml`:

```toml
[patch.crates-io]
hyperlight-component-util = { path = "../src/hyperlight_component_util" }
```

Credit: @squillace identified this fix.
