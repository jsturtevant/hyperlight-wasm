### 2026-03-17: Phase 3.5 WASI-HTTP WIT interface additions
**By:** Billy (Wasm Expert)
**What:** Added `wasi:http@0.2.0` package (types + outgoing-handler) to `hyperlight-sandbox-full.wit`. Used `@0.2.0` version to match existing WIT conventions instead of PRD's `@0.2.3`. World imports added for `wasi:http/types@0.2.0` and `wasi:http/outgoing-handler@0.2.0`.
**Why:** Phase 3.5 networking support — enables Python guests to make outbound HTTP requests via the WASI-HTTP component model interfaces.
