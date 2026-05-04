# Knowledge Base

System-wide learned rules. Read by ALL agents and sessions at startup.
Written ONLY by the auditor after confirming learnings.
Entries are mandatory constraints, not suggestions.

## Provenance Hierarchy
Every entry MUST cite its source using one of:
- `[Source: user override MMDDYY]` — User explicitly corrected something
- `[Source: empirical MMDDYY]` — Verified through testing or data
- `[Source: agent inference MMDDYY]` — Pattern observed by an agent, confirmed by auditor

## Hard Rules
- (none yet — rules accumulate as you work and the auditor validates learnings)

## Platform & Tool Rules
- [043026] When running Rust/Cargo commands via the Bash tool, prepend `~/.cargo/bin` to PATH or use full paths — the Bash tool does not source `~/.cargo/env`. [Source: empirical 043026]

## Project Patterns
- [043026] Default values belong in the parser, not the type. When a command has an optional argument with a default (e.g. `incr foo` defaults to `by: 1`), fill the default in `from_str` so the enum variant is always uniform. Downstream code never branches on "did the user supply this?" [Source: agent inference 043026]
- [043026] Per-arm TDD discipline for `match` dispatchers: only implement an arm when a red test demands it. Leave undemanded arms as a panic stub. The compiler still enforces exhaustiveness; panics on unimplemented arms are loud and informative during development. [Source: agent inference 043026]

## Rust Idioms
- [050126] `?` propagates from the immediately enclosing function only — it cannot escape a closure. Using `?` inside `and_then(|s| ...)` attempts to early-return from the closure, not the outer function. Fix: lift the fallible step out of the closure and use `match` or a let-binding in the function body. [Source: empirical 050126]
- [050126] Pin the target type at the `parse()` site with turbofish (`s.parse::<u64>()`) when inference stalls at a closure boundary. "Type annotations needed" inside a closure usually means the generic type can't be resolved from downstream context; annotating the `parse()` call resolves it. [Source: empirical 050126]
- [050126] `From<E1> for E2` cannot capture calling-scope context — the impl only sees the source error. When an error variant needs to carry context available at the call site (e.g. the bad input the user typed), use `.map_err(|e| Variant { input: s.to_string(), reason: e.to_string() })?` instead of relying on auto-`?`-via-`From`. [Source: agent inference 050126]
- [050126] Asymmetric error variants for asymmetric data sources is idiomatic Rust: `&'static str` for hardcoded labels you control (zero alloc), `String` for user/runtime input, struct variant when multiple pieces of context are needed. Mixing these shapes in one enum is intentional, not inconsistent. [Source: agent inference 050126]
- [050126] Patterns are a separate language from expressions — no function calls, no method calls, no construction inside a pattern. To assert against a complex match arm value: bind the inner data and assert separately (`Err(ParseError::UnknownCommand(s)) => assert_eq!(s, "foo")`). Bind-then-assert produces a diff on failure; a guard (`if`) produces only a generic "didn't match" panic. [Source: empirical 050126]

## Known Failure Modes
- [043026] Rust orphan-file silent failure: a `.rs` file in `src/` not declared via `mod foo;` in `lib.rs` or `main.rs` is silently ignored. `cargo test` reports "0 tests" with no error. Always read the `Running unittests <path>` line in cargo output to confirm the correct crate root. [Source: empirical 043026]
- [050426] `cargo clippy` does not subsume `cargo fmt` — a file can pass clippy and fail `fmt --check` simultaneously. Clippy is a lint check; rustfmt is a layout check. bacon's default jobs do not run fmt. Three-layer prevention: (1) IDE format-on-save, (2) bacon `fmt` job, (3) `.githooks/pre-commit`. CI remains the final catch. [Source: empirical 050426]
- [050426] Line coverage % is dragged down by defensive `panic!` arms in test bodies. These arms only execute when a test fails, so coverage tools count them as 0% covered. Production code coverage is what matters; report the number with that caveat. Do not refactor tests to chase the metric. [Source: empirical 050426]

## Testing Patterns
- [050426] Generic-trait contract test pattern: `pub(crate) mod contract` with test functions generic over the trait (`pub fn foo<S: Trait>(s: &mut S)`), plus a per-impl `macro_rules! delegate!` that generates thin `#[test]` wrappers. New implementations inherit all contract tests by adding one wrapper module — no test rewrites needed. Idiomatic in Rust where each test must be a concrete `#[test] fn`. [Source: agent inference 050426]
