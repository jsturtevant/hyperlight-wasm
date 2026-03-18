# host_bindgen! macro: empty namespace path generates invalid double `::` in codegen

**Repo**: `hyperlight-dev/hyperlight` → `hyperlight-component-util` crate  
**Severity**: Build failure when WIT uses plain interface names without package namespaces  
**Workaround**: Implemented in vendored copy (commit 74fb957, from squillace branch)

## Problem

When a WIT interface name has no package namespace (e.g. a plain `"exports"` interface rather than `wasi:http/types`), the code generator produces invalid Rust paths with a leading `::` separator:

```rust
// Generated (broken):
impl<I: ...> ::Types<...> for ...
// Should be:
impl<I: ...> Types<...> for ...
```

This happens because `wn.namespace_path()` returns an empty `TokenStream` when there are no namespaces, but the codegen unconditionally emits `#tns::#name`, producing `::name`.

## Root Cause

Three locations in the code generator concatenate `tns` (namespace path) with `::` without checking if `tns` is empty:

1. **`rtypes.rs`, `emit_resource_ref`** (~line 124): `quote! { <#sv::#extras #instance_type as #rp #tns::#instance_mod::#rtrait #tvis>::T }`
2. **`rtypes.rs`, `try_find_local_var_id`** (~line 194): `quote! { #rp #tns::#helper::#name }`
3. **`rtypes.rs`, `emit_extern_decl` Instance case** (~line 779): `quote! { type #tn: #rp #tns::#tn #vs; }`
4. **`host.rs`, `emit_export_instance`** (~line 140): `quote! { impl<...> #ns::#trait_name<...> for ... }`

## Suggested Fix

At each location, check if `tns`/`ns` is empty before emitting the `::` separator:

```rust
let trait_ref = if tns.is_empty() {
    quote! { #rp #instance_mod::#rtrait }
} else {
    quote! { #rp #tns::#instance_mod::#rtrait }
};
```

### Files to change:
- `rtypes.rs`: 3 locations (emit_resource_ref, try_find_local_var_id, emit_extern_decl Instance)
- `host.rs`: 1 location (emit_export_instance)
