# Task Board

## Today
- [ ] **Resume first TDD slice on `src/cmd.rs`:** Phase-1 fill in 4 test bodies → confirm 5 real reds → Phase-2 replace `from_str` scaffold → fill in `get` + `incr` arms → confirm 5 greens. Scaffolds are in yesterday's transcript.
- [ ] First commit (user runs `git add` + `git commit`). Drafted message: see yesterday's chat. Don't skip `Cargo.lock`.
- [ ] After 5 greens: review user's `from_str` together; pick next test batch (whitespace, casing, missing-arg, invalid-amount).

## This Week
- [ ] User implements M1 in increments; review together
- [ ] Add rustfmt + clippy config when first code lands
- [ ] Write 1 ADR (Architecture Decision Record) capturing toolchain + early architecture choices
- [ ] Cleanup old auto-memory: `rm -rf ~/.claude/projects/-home-netrom-learn-rust` once verified

## Backlog (Milestone Arc)
- [ ] M2: TCP server + framed protocol (tokio)
- [ ] M3: WAL (Write-Ahead Log) persistence + replay
- [ ] M4: Pub/sub event streaming
- [ ] M5: Consumer groups + queue semantics
- [ ] M6: Single-leader replication
- [ ] M7: Raft-lite leader election (read Raft paper at this milestone)
- [ ] M8: Partitioning/sharding
- [ ] Stretches: snapshots, transactions, gRPC, metrics, client SDK

## Done
- [x] (043026) Pick command surface: `get`/`set`/`del`/`exists`/`incr`/`decr`
- [x] (043026) Pick dispatch: `enum Command` + `match` exhaustiveness
- [x] (043026) Pick storage shape: apply-mutation (Shape B) — `Mutation` + `apply()`
- [x] (043026) Pick module layout: Layout 2 — `lib.rs` + `main.rs` + per-concept files
- [x] (043026) Pick crate name: `nimbus`
- [x] (043026) Rename directory `learn-rust` → `nimbus`; migrate auto-memory
- [x] (043026) Pick error model: per-module; `ParseError` hand-rolled; `StorageError` via `thiserror`; `main.rs` uses `anyhow`
- [x] (043026) Pick REPL input: plain `stdin().read_line()` (with `run()` generic over `BufRead`/`Write` for tests)
- [x] (043026) Pick testing strategy: Heavy + TDD discipline; `proptest` for properties; ~35–50 tests
- [x] (043026) Bootstrap: `cargo init --name nimbus` + `.gitignore`; `Cargo.toml` deps (`thiserror = "2"`, `anyhow = "1"`; dev: `proptest = "1"`, `pretty_assertions = "1"`)
- [x] (043026) CI workflow at `.github/workflows/ci.yml` — fmt --check + clippy -D warnings + test, with `Swatinem/rust-cache@v2`
- [x] (043026) Created `src/lib.rs` (declares `pub mod cmd;`); `main.rs` left as hello-world until REPL exists
- [x] (043026) `src/cmd.rs` scaffolded: `Command` enum (struct variants, all 6 verbs); `ParseError` enum (Empty, UnknownCommand); `impl FromStr` stub; `#[cfg(test)] mod tests` with 1 worked test + 4 skeleton tests (5 placeholder reds)
- [x] (043026) Decided `Incr`/`Decr` use `by: u64` (not `i64`) — principle of least surprise; rejects negatives for free via `s.parse::<u64>()`
