# host_bindgen! macro: name collision when multiple WIT interfaces share the same name

**Repo**: `hyperlight-dev/hyperlight` → `hyperlight-component-util` crate  
**Severity**: Blocking — prevents using WASI-HTTP alongside WASI-filesystem  
**Workaround**: Implemented in vendored copy (commit 74fb957)

## Problem

When a WIT component imports multiple interfaces that share the same final name from different packages (e.g. `wasi:filesystem/types` and `wasi:http/types`), the `host_bindgen!` macro generates duplicate trait members in the `RootImports` trait:

```rust
// Both produce:
type Types = ...;
fn types(&mut self) -> ...;
```

This causes: `error[E0428]: the name 'Types' is defined multiple times`

## Root Cause

In `hyperlight-component-util/src/emit.rs`, `kebab_to_type()` and `kebab_to_getter()` only receive the final interface name (e.g. `"types"`), ignoring the parent namespace. Three code paths are affected:

1. **Trait member generation** (`rtypes.rs`, `emit_extern_decl` Instance case) — generates `type #tn` and `fn #getter`
2. **Host function registration** (`host.rs`, `emit_import_extern_decl` Instance case) — generates getter/type for `SelfInfo::with_getter`
3. **Type variable naming** (`emit.rs`, `noff_var_id`) — resolves bound type variables using the colliding name

## Suggested Fix

Add a `colliding_import_names: HashSet<String>` field to `State`. Before iterating imports in `emit_component`, scan for name collisions. When a collision exists, prepend the parent namespace:

- `wasi:filesystem/types` → `FilesystemTypes` / `filesystem_types()`
- `wasi:http/types` → `HttpTypes` / `http_types()`

Non-colliding names remain unchanged.

### Files to change:
- `emit.rs`: Add `colliding_import_names` to `State`, add `find_colliding_import_names()` and `import_member_names()` helpers
- `rtypes.rs`: Use `import_member_names()` in Instance case of `emit_extern_decl`, and in `emit_resource_ref`
- `host.rs`: Use `import_member_names()` in Instance case of `emit_import_extern_decl`

## Reproduction

```wit
world root {
  import wasi:filesystem/types@0.2.0;
  import wasi:http/types@0.2.0;
  export hyperlight:sandbox/executor;
}
```
