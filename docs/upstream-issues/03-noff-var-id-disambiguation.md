# host_bindgen! macro: noff_var_id doesn't account for disambiguated interface names

**Repo**: `hyperlight-dev/hyperlight` → `hyperlight-component-util` crate  
**Severity**: Build failure — generates `I::Types` reference when trait member was renamed to `I::FilesystemTypes`  
**Workaround**: Implemented in vendored copy (commit 74fb957)  
**Related**: Depends on issue #01 (name collision fix)

## Problem

When the name collision fix (issue #01) renames trait members from `Types` to `FilesystemTypes`, the `noff_var_id` function in `emit.rs` still generates the old `Types` name. This causes:

```
error[E0220]: associated type `Types` not found for `I`
```

The resource table struct and the `emit_resource_ref` function both use `noff_var_id` to name type variables, so they generate `I::Types` when the trait now expects `I::FilesystemTypes`.

## Root Cause

`noff_var_id` (`emit.rs`, ~line 476) extracts the variable name from `TyvarOrigin::last_name()`, which returns the raw WIT interface name (e.g. `"types"`). It then calls `kebab_to_type()` which produces `Types` — but the trait member was renamed to `FilesystemTypes` by the collision fix.

Similarly, `emit_resource_ref` (`rtypes.rs`, ~line 96) uses `kebab_to_type(iwn.name)` for the `instance_type` variable used in `<I::instance_type as ...>::T` expressions.

## Suggested Fix

Both `noff_var_id` and `emit_resource_ref` need to use the same disambiguation logic as the trait member generation:

```rust
// In noff_var_id:
let wn = split_wit_name(name);
let (tn, _) = import_member_names(&wn, &self.colliding_import_names);

// In emit_resource_ref, for imported instances:
let instance_type = if path[path.len() - 2].imported() {
    let (tn, _) = import_member_names(&iwn, &s.colliding_import_names);
    tn
} else {
    kebab_to_type(iwn.name)
};
```

### Files to change:
- `emit.rs`: `noff_var_id` function
- `rtypes.rs`: `emit_resource_ref` function
