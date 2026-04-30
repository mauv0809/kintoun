# Task Board

## Today
- [x] Pick command surface: `get`/`set`/`del`/`exists`/`incr`/`decr`
- [x] Pick dispatch: `enum Command` + `match` exhaustiveness
- [x] Pick storage shape: apply-mutation (Shape B) — `Mutation` + `apply()`
- [x] Pick module layout: Layout 2 — `lib.rs` + `main.rs` + per-concept files
- [x] Pick crate name: `nimbus`
- [x] Rename directory `learn-rust` → `nimbus`; migrate auto-memory
- [x] Pick error model: per-module; `ParseError` hand-rolled; `StorageError` via `thiserror`; `main.rs` uses `anyhow`
- [x] Pick REPL input: plain `stdin().read_line()` (with `run()` generic over `BufRead`/`Write` for tests)
- [x] Pick testing strategy: Heavy + TDD discipline; `proptest` for properties; ~35–50 tests
- [x] **Bootstrap:** `cargo init --name nimbus` + `.gitignore` (cargo + Claudify locals) — done; first commit still **pending (user runs)**
- [x] Add deps to `Cargo.toml`: `thiserror = "2"`, `anyhow = "1"`; dev: `proptest = "1"`, `pretty_assertions = "1"` — `cargo build` + `cargo test` green (0 tests)
- [x] CI workflow created: `.github/workflows/ci.yml` — fmt --check + clippy -D warnings + test, with `Swatinem/rust-cache@v2`
- [ ] **First TDD slice (active):** write 1–3 failing parser tests in `src/cmd.rs`, then stub `Command` + `ParseError` to compile, then minimum code to green
- [ ] Cleanup old auto-memory: `rm -rf ~/.claude/projects/-home-netrom-learn-rust` once verified

## This Week
- [ ] User implements M1 in increments; review together
- [ ] Add rustfmt + clippy config when first code lands
- [ ] Stand up minimal GitHub Actions CI (fmt + clippy + test) once M1 has tests
- [ ] Write 1 ADR (Architecture Decision Record) capturing toolchain + early architecture choices

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
-
