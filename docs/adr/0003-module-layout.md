# ADR 0003: Module Layout — lib.rs + main.rs Split, One Concept Per File

Date: 2026-04-30
Status: Accepted

## Context

A Rust binary crate has multiple legal source layouts. The choice affects:
- **Testability** — integration tests under `tests/` can only target a library, not a binary.
- **Reusability** — a library can be depended on by other crates; a binary cannot.
- **Clarity** — where do tests live, where do public APIs live, where does the binary entry point live.

This decision was made up front during the M1 design pass. Switching layouts later would force churn across every module's `mod` and `use` declarations.

Several layouts were weighed during the design pass; the alternatives were rejected directionally rather than via measurement, and their specifics are not preserved in working notes. The selected layout is referred to internally as "Layout 2."

## Decision

Use the **`lib.rs` + `main.rs` split**, with one concept per file:

```
src/
├── lib.rs        # crate library entry; declares pub modules
├── main.rs       # binary entry; thin shim calling kintoun::repl::run(...)
├── cmd.rs        # command parsing (Command enum, ParseError, FromStr impl)
├── storage.rs    # Storage trait + InMemoryStorage + Mutation/MutationOutcome
├── executor.rs   # turning Commands into Storage operations
├── repl.rs       # interactive loop, generic over BufRead + Write
└── error.rs      # cross-cutting error types if needed

tests/
└── kv_integration.rs   # optional end-to-end integration tests via the library
```

`main.rs` stays minimal. It constructs concrete IO (`stdin().lock()`, `stdout().lock()`) and hands them to the library's `repl::run(...)`. All real logic lives in the library.

## Consequences

- Integration tests in `tests/` target the library directly and can exercise the public API surface. A pure-binary layout would force end-to-end tests to spawn the binary as a subprocess.
- The crate is reusable as a library. A future `kintoun-server` binary, a `kintoun-cli` binary, or Rust-side embedded use can all depend on `kintoun` directly without re-implementing.
- Every new module must be declared in `lib.rs` with `pub mod foo;`. Forgetting this leaves the file orphaned — `cargo test` reports zero tests for that module with no error. The fix is mechanical once spotted; the failure is silent until then.
- "One concept per file" trades larger-file simplicity for more-file count. As of M1's design lock, this means 5-6 source files in `src/` instead of one large `lib.rs`. Worth it for navigation and per-file ownership of concerns.
- The `tests/kv_integration.rs` file is optional, not required. Per-module unit tests under `#[cfg(test)] mod tests` (inside each source file) handle most coverage; integration tests are reserved for end-to-end flows that span modules.
