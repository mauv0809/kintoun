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

## Known Failure Modes
- [043026] Rust orphan-file silent failure: a `.rs` file in `src/` not declared via `mod foo;` in `lib.rs` or `main.rs` is silently ignored. `cargo test` reports "0 tests" with no error. Always read the `Running unittests <path>` line in cargo output to confirm the correct crate root. [Source: empirical 043026]
