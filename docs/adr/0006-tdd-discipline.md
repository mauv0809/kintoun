# ADR 0006: Testing Strategy — Heavy TDD, Per-Arm Discipline

Date: 2026-04-30
Status: Accepted

## Context

For this Rust learning project, the testing strategy serves two simultaneous purposes:

1. **Correctness.** The conventional point. Tests catch regressions, especially across milestone transitions where the interface surface area churns.
2. **Pedagogy.** Heavy testing exposes the user to Rust's testing idioms: `#[cfg(test)]`, `#[test]`, `assert_eq!`, `match` arms in tests, property tests via `proptest`, generic IO for testability.

A light test suite would underdeliver on both. A heavy suite costs feature velocity but pays dividends through milestone transitions, where regressions in earlier modules become expensive once those modules are taken for granted.

## Decision

- **Heavy test coverage.** Target ~35-50 tests across parser/storage/executor/repl by end of M1, plus property tests where appropriate. As of 2026-05-02, `cmd.rs` alone has 28 tests.
- **Red-green-refactor TDD.** Write a failing test first, make it pass, refactor. The user writes test bodies; Claude reviews and provides scaffolding for the implementation arm.
- **Per-arm TDD discipline.** For `match` dispatchers (e.g. the verb match in `cmd::FromStr`), implement an arm only when a red test demands it. Undemanded arms stay as panic stubs. The compiler enforces exhaustiveness; panics on unreached arms are loud and informative during development.
- **Per-layer test-writing split.** When a new layer (parser, storage, executor, repl) lands, the user writes the first 2-3 tests by hand to learn Rust testing idioms. Claude expands the suite from there. The user implements to green.
- **Generic REPL signature for testability.** `repl::run<R: BufRead, W: Write>(...)` — generic over input reader and output writer. Tests pass `Cursor<&[u8]>` and `Vec<u8>`; `main.rs` constructs `stdin().lock()` and `stdout().lock()`.
- **Storage trait test suite as contract.** The `Storage` trait will get a generic test suite (parameterized over implementor) that doubles as the contract M3+ implementations must satisfy. When a WAL-backed storage impl lands in M3, it must pass the same tests `InMemoryStorage` passes.
- **Dev dependencies:** `proptest = "1"` for property tests, `pretty_assertions = "1"` for readable diffs on assertion failures.

## Consequences

- The test suite catches regressions across milestone transitions. When M3 introduces a WAL-backed `Storage` impl, the same contract tests guard behavior parity with `InMemoryStorage`.
- Feature velocity is slower than a no-test approach. This is intentional. The project's primary outcome is learning Rust, not shipping fast.
- Per-arm TDD prevents speculative implementation — code only exists when a red test forces it. Combined with `match` exhaustiveness, this gives strong guarantees that every code path in production is exercised by at least one test.
- Generic REPL signature means tests don't touch real stdin/stdout. CI doesn't need TTY emulation. The cost: a slightly more complex function signature than `fn run()` would have.
- Per-layer test-writing split slows the user down on each layer's first few tests but builds testing literacy that compounds across layers. After parser, the user needed less hand-holding for storage/executor/repl test idioms.
- `pretty_assertions` is a pure quality-of-life upgrade — failed assertions now produce colored diffs rather than `expected: ... actual: ...` runs. No runtime cost in non-test builds.
