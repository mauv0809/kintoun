# ADR 0011: Test Coverage Tooling — cargo-llvm-cov

Date: 2026-05-04
Status: Accepted

## Context

ADR 0006 establishes heavy TDD with per-arm discipline for `match` dispatchers. That gives strong assurance for dispatchers — every arm has a test that drove its implementation — but does not directly cover branches *within* helper functions, error-propagation paths through deeper layers, or, most importantly, *new* code added by future contributors who may not honor the discipline.

The visibility question came up explicitly during M1 close. The honest answer: per-arm TDD is necessary but not sufficient. A coverage tool surfaces gaps that discipline misses, and provides a number we can ratchet over time as a regression gate.

### Alternatives Considered

**A — `cargo-tarpaulin`.** Original Rust coverage tool. Linux-native via ptrace. Well-known; widely tutorialized. Approximate branch coverage. Pre-LLVM-coverage era. Rejected: cross-platform story is weaker, branch coverage is approximate, and the modern alternative (B) is now the de-facto choice.

**B — `cargo-llvm-cov` (chosen).** Wraps `rustc`'s built-in `-Cinstrument-coverage` flag — the same LLVM-based machinery the Rust team uses internally. Cross-platform, accurate branch coverage, outputs lcov / HTML / JSON / terminal summary. Maintained by `taiki-e`, who also publishes `taiki-e/install-action` for one-line CI installation.

**C — No coverage tool, rely on TDD discipline.** Original M1 stance, reversed mid-session. Discipline is a forcing function on developers; coverage is a verifier on the result. They serve different purposes. Running only TDD leaves a gap when discipline slips, and provides no number to ratchet.

## Decision

1. **Tool: `cargo-llvm-cov`.** Installed in CI via `taiki-e/install-action@cargo-llvm-cov`. Replaces the existing `cargo test` step — `cargo llvm-cov` runs the test suite under instrumentation, so the two operations collapse into one step (no double test execution).

2. **Output: terminal summary (`--summary-only`).** A coverage table printed in the CI log: per-file lines/regions/functions covered. Sufficient visibility for M1 without external service integration.

3. **No threshold enforcement at M1.** Today's number is unknown. Setting a threshold before measuring would either bind us to whatever we happen to have, or require a placeholder we then have to update. Approach: *measure first*, watch the number for a few runs, ratchet up once we have a stable baseline.

4. **Ratchet plan: revisit at M3.** When the WAL backend lands, set a threshold based on observed M1+M2+M3 coverage. Likely 80% lines, with branch coverage as a softer target. The contract test suite (ADR 0006 point D) helps: M3's `WalBackedStorage` inherits the 22 storage contract tests automatically, keeping coverage of the storage layer high without per-impl effort.

5. **Property-based testing as a complement.** ADR 0006 specified `proptest` as a dev-dependency. M1 close added the first two property tests on `StoredValue::from_text` (round-trip on success path; fall-through on failure path). Coverage and property tests answer different questions:
   - Coverage: "which lines ran during testing?"
   - Property: "does the invariant hold across the input space?"

   Both useful; neither replaces the other.

## Consequences

- Every PR runs `cargo llvm-cov` in CI and surfaces a coverage summary table in the log. Numbers are visible without enforcement.
- CI cost: roughly 20-40 seconds added per run. Negligible against the existing test job, especially since `cargo llvm-cov` *replaces* `cargo test` rather than running on top of it.
- The ratchet plan turns coverage into a forcing function over time. Once a threshold is set, regressions fail CI, making "I tested it" require literal evidence rather than goodwill.
- `cargo-llvm-cov` works locally — `cargo llvm-cov --html` opens a browsable per-file report. Useful when investigating "why did coverage drop?" or finding a specific untested path.
- The contract test suite (ADR 0006 point D) compounds with coverage tooling: M3's `WalBackedStorage` will start with the storage layer's coverage already high because the contract is shared. Both reward the structural test choice.
- The toolchain step in CI now needs `llvm-tools-preview` as a component (cargo-llvm-cov requires LLVM coverage utilities). One-line addition to the existing `dtolnay/rust-toolchain@stable` step.

## Open Follow-ups

- **Threshold ratchet at M3.** Once the WAL backend lands, set `--fail-under-lines N` based on observed coverage. Likely 75-80% to start, with room to climb.
- **Branch coverage flag.** `cargo llvm-cov --branch` enables branch coverage; off by default. Consider once line coverage is high and the next leverage point is branches.
- **Codecov / Coveralls integration.** Useful for PR-level coverage diff comments ("this PR drops coverage by 1.2%"). Adds an external service dependency. Defer until the project goes public-facing or the team grows.
- **Mutation testing.** `cargo-mutants` is a stronger signal than coverage — does removing this line make any test fail? Heavier to run, more meaningful when it does. Defer past M3.
