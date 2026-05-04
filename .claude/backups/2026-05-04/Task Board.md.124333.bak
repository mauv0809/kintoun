# Task Board

## Today
- [ ] **`storage.rs` stopping point B** — first `impl Storage for InMemoryStorage` block; wire `Set` mutation + `Get` read; 2-4 unit tests; per-arm TDD. ~30-45 min. Will self-clean the `#[expect(dead_code)]` on `data` field.

## This Week
- [ ] `storage.rs` stopping point C — full verb coverage (Del, Exists, Incr, Decr); locks 5 open behavior decisions via TDD
- [ ] `storage.rs` stopping point D — generic `Storage` contract test suite (ADR 0006)
- [ ] `executor.rs` — turn `Command`s into `Storage` operations
- [ ] `repl.rs` — interactive loop, generic over `BufRead + Write`
- [ ] Wire `main.rs` as thin shim calling `kintoun::repl::run(...)`
- [ ] Cleanup: `rm -rf ~/.claude/projects/-home-netrom-{nimbus,learn-rust}` (verified migrated)
- [ ] Cleanup: `rm -rf .claude/skills/gitnexus` and `rm -rf ~/.claude/hooks/gitnexus` (dormant after hook removal)

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
- [x] **2026-05-02:** Parser case-insensitivity (`to_ascii_lowercase()` + 2 tests for uppercase/mixed-case verbs)
- [x] **2026-05-02:** Whitespace edge-case tests (6 tests: leading, trailing, multiple, tabs, newline, all-whitespace)
- [x] **2026-05-02:** Trailing-args rejection — `ParseError::TooManyArgs(&'static str)` + `expect_done` helper + 6 tests
- [x] **2026-05-02:** Original-case verb preserved in `UnknownCommand`; test added
- [x] **2026-05-02:** `ParseError` Display+Error: hand-rolled then migrated to `#[derive(thiserror::Error)]` (educational pass complete)
- [x] **2026-05-02:** Backfilled `del`/`exists`/`decr` parser arms with TDD cycles (full M1 verb coverage)
- [x] **2026-05-02:** Wrote 7 ADRs in `docs/adr/` (Nygard format) — 0001 toolchain, 0002 crate name, 0003 module layout, 0004 error model, 0005 storage shape, 0006 TDD, 0007 grammar
- [x] **2026-05-02:** ADR 0005 expanded with reconstructed Shape A alternatives section
- [x] **2026-05-02:** README — added Architecture section summarizing mutation/read split, pointing at ADR 0005
- [x] **2026-05-02:** Created `rustfmt.toml` (`edition = "2024"`, `max_width = 100`) and `clippy.toml` (`msrv = "1.85"`)
- [x] **2026-05-02:** Auto-memory migration `~/.claude/projects/-home-netrom-nimbus` → `-home-netrom-kintoun`; project file renamed `project_nimbus.md` → `project_kintoun.md`
- [x] **2026-05-02:** GitNexus removed — CLAUDE.md block, `.gitnexus/` index, `.gitignore` line, user-global PreToolUse + PostToolUse hooks
- [x] **2026-05-02:** Stop hook stabilized — model alias troubleshooting; finally disabled prompt step (kept logger script)
- [x] **2026-05-02:** `storage.rs` **stopping point A** — `Mutation`, `MutationOutcome`, `StorageError`, `Storage` trait, `InMemoryStorage` skeleton; `pub mod storage;` in `lib.rs`; cargo check + clippy clean (with `#[expect(dead_code)]` on `data`)
