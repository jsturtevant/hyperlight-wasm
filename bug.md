## `wasm_guest_bindgen!` macro doesn't support WIT `flags` types (needed for `wasi:filesystem`)

The `wasm_guest_bindgen!` macro in `hyperlight_wasm_macro` fails to compile when the WIT world includes `flags` type definitions (e.g. from `wasi:filesystem/types@0.2.0`).

### Error

```
error[E0277]: the trait bound `DescriptorFlags: wasmtime::component::Lower` is not satisfied
error[E0277]: the trait bound `PathFlags: wasmtime::component::Lift` is not satisfied
error[E0277]: the trait bound `OpenFlags: wasmtime::component::Lift` is not satisfied
```

### Root cause

The macro generates `#[derive(Lift, Lower)]` with `#[component(record)]` for type definitions (line ~313 in `wasmguest.rs`), but WIT `flags` types need `#[component(flags)]` — not `#[component(record)]`. The `flags` case is not handled in the type emission codepath in `hyperlight-component-util`.

### WIT that triggers the error

```wit
flags descriptor-flags {
  read,
  write,
  file-integrity-sync,
  data-integrity-sync,
  requested-write-sync,
  mutate-directory,
}

flags path-flags {
  symlink-follow,
}

flags open-flags {
  create,
  directory,
  exclusive,
  truncate,
}
```

These are standard `wasi:filesystem/types@0.2.0` definitions — required by any guest component built with `componentize-py` (CPython imports the full WASI filesystem interface).

### Impact

Without `flags` support, it's impossible to use `wasi:filesystem` with hyperlight-wasm component model support. This blocks:

- File I/O via WASI preopens (the filesystem `descriptor` resource methods like `open-at`, `get-flags`, `stat-at` all use `flags` types)
- Any guest component that imports `wasi:filesystem/types` (including all `componentize-py` Python components built without `--stub-wasi`)

The `hyperlight-wasm-sockets-example` works around this by defining a minimal `wasi:filesystem/types` interface that omits all methods using `flags`, but this means the runtime linker rejects guest components that import those methods.

### Expected fix

In the type codegen (likely `hyperlight-component-util`'s type emission), add a case for WIT `flags` definitions that generates:

```rust
#[derive(wasmtime::component::ComponentType)]
#[derive(wasmtime::component::Lift)]
#[derive(wasmtime::component::Lower)]
#[component(flags)]
struct DescriptorFlags {
    #[component(name = "read")] read: bool,
    #[component(name = "write")] write: bool,
    // ...
}
```

This is the standard wasmtime pattern for `flags` — each flag becomes a `bool` field with `#[component(flags)]` instead of `#[component(record)]`.

### Reproduction

1. Create a WIT with `wasi:filesystem/types` including `flags descriptor-flags { ... }`
2. Compile with `wasm-tools component wit ... -w -o world.wasm`
3. Build with `WIT_WORLD=world.wasm cargo build -p hyperlight-wasm`
4. Runtime sub-build fails with the `Lift`/`Lower` errors above

### Workaround

Strip all `flags` types and methods that use them from the WIT. This prevents using filesystem operations that require flags parameters.

### References

- `hyperlight-wasm-sockets-example` — works around by omitting `flags` from WIT
- `hyperlight-wasm-http-example` — avoids the issue by not importing `wasi:filesystem` at all
- wasmtime flags derive docs: https://docs.rs/wasmtime/latest/wasmtime/component/derive.ComponentType.html
