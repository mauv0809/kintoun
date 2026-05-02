# Task Board

## Today
- [ ] **Hand-roll `Display` + `std::error::Error` for `ParseError`** — yesterday's M1 design's "educational pass" (see the trait once before reaching for `thiserror`). Currently `ParseError` only derives `Debug` + `PartialEq`. ~20 min, no tests change.
- [ ] **Backfill `del` / `exists` / `decr` arms** — three near-identical TDD cycles (test → arm). ~15 min. Locks full verb coverage.
- [ ] **Edge-case parser tests** — whitespace runs, casing (`SET` vs `set`), trailing args, multi-word values for `set`. May surface new variants or impl tweaks.
- [ ] **OR move on to `storage.rs`** — `Storage` trait + `InMemoryStorage` over `HashMap` + apply-mutation pattern (`Mutation` + `MutationOutcome`). New territory: traits with associated types. ~30–60 min.

## This Week
- [ ] Continue M1 in increments; review together
- [ ] Add `rustfmt.toml` + `clippy.toml` configs (currently CI runs `cargo fmt --check` against defaults)
- [ ] Write 1 ADR (Architecture Decision Record) capturing toolchain + early architecture choices
- [ ] Cleanup old auto-memory: `rm -rf ~/.claude/projects/-home-netrom-learn-rust` and `-home-netrom-nimbus` once `-home-netrom-kintoun/` is verified working

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
(Friday EOD 050126: cleared. Full history in `Daily Notes/`.)
