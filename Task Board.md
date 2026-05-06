# Task Board

## Today
- [ ] **M3 kickoff prep** — read DDIA ch.3 (Storage and Retrieval). Then run a design pass on WAL format (line-delimited text vs binary record), append + fsync policy, replay on startup, and how `WalBackedStorage` plugs into the existing `Storage` trait + contract test suite.

## This Week
- [ ] **M3 design pass + ADR** — once the WAL direction clarifies, write the ADR (Nygard format; thicker Alternatives section since the durability decision shapes M4–M6).

## Backlog (Milestone Arc)
- [ ] M3: WAL (Write-Ahead Log) persistence + replay
- [ ] M4: Pub/sub event streaming
- [ ] M5: Consumer groups + queue semantics
- [ ] M6: Single-leader replication
- [ ] M7: Raft-lite leader election (read Raft paper at this milestone)
- [ ] M8: Partitioning/sharding
- [ ] Stretches: snapshots, transactions, gRPC, metrics, client SDK
- [ ] Coverage threshold ratchet at M3 (per ADR 0011)
- [ ] Graceful drain timeout (ADR 0013 follow-up; revisit at M5+ when long-running consumer connections appear)
- [ ] Post-M1 quoting + Str escape rules + inference policy

## Done
- [x] **2026-05-06 (PM): M2 FEATURE-COMPLETE** — `serve(listener, storage, shutdown: F)` with `JoinSet` drain (per-conn tasks tracked; on shutdown listener drops then `join_next` until empty; no drain timeout). `tests/tcp_integration.rs` — 6 real-TCP tests (round-trip, multi-client shared, oversize frame teardown, partial-prefix EOF, pipelined order, concurrent INCR via Mutex). `main.rs` rewritten — hand-rolled `parse_args` (Mode = Repl | Server(SocketAddr)) + 6 unit tests; lazy multi-thread tokio runtime on server branch only; `tokio::signal::ctrl_c()` wrapped to `Future<Output = ()>`. ADR 0013 status note added (flat-file module layout, serve signature, JoinSet drain, lazy runtime captured as deltas). End-to-end smoke green: real TCP SET → OK; SIGINT → exit 0. **129 tests total**.
- [x] **2026-05-06 (AM):** **M2 connection layer shipped** (commit b14cf05) — `handle_connection<IO, S>` async per-connection task; `Arc<Mutex<Storage>>` shared state with `std::sync::Mutex` block-scoped guard (compile-time `!Send` footgun protection); error policy locked (codec/utf-8/write fatal; parse/executor → `ERR <msg>`); `format_error` helper added; 7 duplex tests; `futures = "0.3"` dep added. ~117 tests total.
- [x] **2026-05-04 (EOD):** **M2 codec layer + format extraction shipped** — 4 commits: ADRs 0012/0013, codec + format module, session bookkeeping, `.claude/backups/` gitignore. 14 codec tests (13 unit + proptest) + 10 format tests. ~110 tests total green.
- [x] **2026-05-04:** **ADR 0012** — M2 wire protocol (TCP + length-prefix envelope + text payload).
- [x] **2026-05-04:** **ADR 0013** — M2 server architecture (sync REPL + async server alongside, Arc<Mutex>, three-layer testing).
- [x] **2026-05-04:** **`src/format.rs`** extracted from `repl.rs` — shared format module for REPL + TCP server. 10 unit tests.
- [x] **2026-05-04:** **`src/server/codec.rs`** — length-prefixed FrameCodec. 13 unit tests + proptest.
- [x] **2026-05-04:** Cargo deps added — tokio, tokio-util, bytes. `.gitignore` updated for `.claude/backups/`.
- [x] **2026-05-04:** **M2 design pass complete** — six decisions locked (transport, wire format, architecture, module layout, testing, error boundary).
- [x] **2026-05-04:** Cleanup — removed `~/.claude/projects/-home-netrom-{nimbus,learn-rust}` (migration verified); confirmed `.claude/skills/gitnexus` and `~/.claude/hooks/gitnexus` already absent.
- [x] **2026-05-04:** **M1 STRUCTURALLY COMPLETE** — kintoun runs interactively (`cargo run`). 86 tests; clippy + fmt clean; CI green.
- [x] **2026-05-04:** `repl.rs` — generic over BufRead+Write+Storage, anyhow error unification at boundary, Redis-like format, 11 tests (loop semantics + format precision)
- [x] **2026-05-04:** `main.rs` wired as 5-line shim (locks stdin/stdout, builds InMemoryStorage, calls `repl::run`)
- [x] **2026-05-04:** ADR 0010 — REPL output format + thiserror-vs-anyhow boundary convention
- [x] **2026-05-04:** Generic Storage contract test suite (ADR 0006 point D) — 22 contract fns + `macro_rules! delegate!` wrappers; M3+ backends inherit by adding one wrapper module
- [x] **2026-05-04:** First proptest properties on `StoredValue::from_text` — round-trip + fall-through across the full input space (~256 cases each per CI run)
- [x] **2026-05-04:** ADR 0011 + `cargo-llvm-cov` in CI (replaces `cargo test` step; no threshold yet — measure first, ratchet at M3)
- [x] **2026-05-04:** Local fmt parity infrastructure — `bacon.toml` fmt job, `.githooks/pre-commit`, `docs/dev-setup.md` (multi-IDE format-on-save). Caught after CI fmt failure on the M1-polish commit.
- [x] **2026-05-04:** `storage.rs` stopping points B + C — full mutations/reads with all 5 behavior decisions locked via TDD; `StoredValue` enum with `Str`/`Int` variants
- [x] **2026-05-04:** `StoredValue::from_text` — text→type inference at executor boundary; 8 inference tests covering numeric/non-numeric/negative/decimal/zero/u64::MAX/overflow/empty boundaries
- [x] **2026-05-04:** 2 `del-missing` idempotency tests in storage
- [x] **2026-05-04:** ADR 0005 status-update + ADR 0008 (tagged `StoredValue` + executor-level inference + A/B/C/D alternatives section)
- [x] **2026-05-04:** ADR 0009 + `deny.toml` + cargo-deny CI step (permissive license allowlist; `unused-allowed-license = "allow"`)
- [x] **2026-05-04:** `executor.rs` — all 6 arms wired with `apply_and_wrap` helper, structured `ExecuteResult` (Mutation/Read/Existence), `ExecuteError` with `#[from] StorageError`; 15 tests covering round-trips, inference paths, error propagation (NotAnInteger, Underflow, Overflow), keystone test proving the layered design
- [x] **2026-05-02:** Parser case-insensitivity (`to_ascii_lowercase()` + 2 tests for uppercase/mixed-case verbs)
- [x] **2026-05-02:** Whitespace edge-case tests (6 tests: leading, trailing, multiple, tabs, newline, all-whitespace)
- [x] **2026-05-02:** Trailing-args rejection — `ParseError::TooManyArgs(&'static str)` + `expect_done` helper + 6 tests
- [x] **2026-05-02:** Original-case verb preserved in `UnknownCommand`; test added
- [x] **2026-05-02:** `ParseError` Display+Error: hand-rolled then migrated to `#[derive(thiserror::Error)]` (educational pass complete)
- [x] **2026-05-02:** Backfilled `del`/`exists`/`decr` parser arms with TDD cycles (full M1 verb coverage)
- [x] **2026-05-02:** Wrote 7 ADRs in `docs/adr/` (Nygard format) — 0001 toolchain, 0002 crate name, 0003 module layout, 0004 error model, 0005 storage shape, 0006 TDD, 0007 grammar
- [x] **2026-05-02:** ADR 0005 expanded with reconstructed Shape A alternatives section
- [x] **2026-05-02:** README — added Architecture section summarizing mutation/read split, pointing at ADR 0005 (later: also 0008)
- [x] **2026-05-02:** Created `rustfmt.toml` (`edition = "2024"`, `max_width = 100`) and `clippy.toml` (`msrv = "1.85"`)
- [x] **2026-05-02:** Auto-memory migration `~/.claude/projects/-home-netrom-nimbus` → `-home-netrom-kintoun`
- [x] **2026-05-02:** GitNexus removed — CLAUDE.md block, `.gitnexus/` index, `.gitignore` line, user-global PreToolUse + PostToolUse hooks
- [x] **2026-05-02:** Stop hook stabilized — model alias troubleshooting; finally disabled prompt step (kept logger script)
- [x] **2026-05-02:** `storage.rs` **stopping point A** — design surface
