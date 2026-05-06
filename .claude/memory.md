# Memory

## Project
- **Name:** **kintoun** (crate + directory + GitHub repo). Directory: `/home/netrom/kintoun`. `kintoun = 筋斗雲`, Goku's Flying Nimbus. Future cloud version: `kintoun.cloud`.
- **What:** Distributed KV store with stream/queue/cluster ambitions, built as the vehicle for learning Rust.
- **Language/build tool:** Rust 2024, Cargo. MSRV 1.85. Toolchain at `~/.cargo/bin` — Bash tool doesn't source `~/.cargo/env`; prepend to PATH or use full paths.
- **State (2026-05-06):** **M2 feature-complete.** Real-TCP server with framed wire protocol; REPL (no flag) or server (`--bind <addr>`) dispatch; Ctrl-C → JoinSet drain. **129 tests green** (M1 86 + codec 14 + format 10 + connection 7 + tcp_integration 6 + main 6); 13 ADRs; clippy + fmt clean. Next: M3 (WAL + replay).
- **Persona/niche:** Deferred. Decide ~M4 once friction surfaces real angles.

## Workflow Rules (load every session)
- **Pseudocode-first** + **substantial starter kits for Rust beginners.** User adapts and re-types into their files. They write the meaningful logic; I provide scaffolding.
- **Loop:** design together → user codes → review together. Don't race ahead.
- **Frame Rust against other languages** the user already knows.
- **Always explain acronyms on first use** (REPL, WAL, RPC, etc.) until told to stop.
- **Hold the line on milestone scope.** Only forward-looking constraint: "don't make M1 decisions that block M4–M8."
- **Push back when substance warrants.** Concede only when the user's argument actually moves a premise; don't capitulate to social pressure.

## Milestone Arc (locked 2026-04-30)
- M1: In-memory KV + REPL ✅ **structurally complete 2026-05-04**
- M2: TCP server + framed protocol (tokio) ✅ **feature-complete 2026-05-06**
- M3: WAL persistence + replay ← **next**
- M4: Pub/sub event streaming on the log
- M5: Consumer groups + offsets (queue semantics)
- M6: Single-leader async replication
- M7: Raft-lite leader election
- M8: Partitioning/sharding
- **Deferred surfaces:** gRPC client-facing protocol (layered over the same executor/storage as a parallel surface to raw TCP); client SDK; snapshots; transactions; metrics. Already on Task Board Stretches. Full TCP-vs-gRPC reasoning to be captured in the M2 protocol ADR's Alternatives Considered section when written.

## M1 Module Status
- `cmd.rs` ✅ feature-complete (29 tests, 95% line coverage)
- `storage.rs` ✅ all mutations + reads + `from_text` + generic `contract` test module + 2 proptest properties (33 tests, 100% coverage)
- `executor.rs` ✅ all 6 arms wired with `apply_and_wrap` helper (14 tests, 100% coverage)
- `repl.rs` ✅ generic over BufRead+Write+Storage, anyhow error unification, Redis-like format (11 tests)
- `main.rs` ✅ 5-line shim — locks stdin/stdout, builds InMemoryStorage, calls repl::run
- `tests/kv_integration.rs` ❌ optional; lands when M2 starts touching multiple layers

## M2 Design (locked 2026-05-04, ADRs 0012 + 0013)
- **Transport:** TCP, default bind `127.0.0.1:4242`, `--bind <addr:port>` override (hand-rolled CLI parsing in `main.rs`); Ctrl-C graceful shutdown.
- **Wire format:** length-prefixed envelope `[len:u32 BE][payload]`; payload = UTF-8 text command line; M1's `cmd::parse` reused unchanged.
- **Architecture:** sync REPL stays; new async server alongside; both share `cmd`/`executor`/`storage`. Tokio task-per-connection with `Arc<Mutex<InMemoryStorage>>` (RwLock/sharding deferred).
- **Module layout:** `src/server.rs` (Rust 2018+ flat-file style) + `src/server/{codec,connection}.rs`.
- **Tests:** three layers — codec unit tests (sync), in-memory `tokio::io::duplex` connection tests, real-TCP integration in `tests/tcp_integration.rs`.
- **Errors:** anyhow at server boundary, thiserror at typed layers, `ERR <msg>` payload to client (consistent with REPL per ADR 0010).
- **Future evolution:** add `frame_type:u8` byte to envelope at M4 when push frames arrive; format-negotiation handshake only if ever needed.

## M2 Module Status (all ✅ as of 2026-05-06)
- `cmd.rs`, `executor.rs`, `storage.rs` reused unchanged from M1
- `format.rs` extracted from `repl.rs` (10 unit tests, all 8 ExecuteResult arms + boundary edges + `format_error`)
- `server.rs` flat-file: `pub mod codec; pub mod connection;` + `serve(listener, storage, shutdown)` with JoinSet drain
- `server/codec.rs` length-prefixed `FrameCodec` (13 unit tests + 1 proptest property)
- `server/connection.rs` `handle_connection<IO, S>` generic over AsyncRead+AsyncWrite+Unpin and Storage+Send; `std::sync::Mutex` block-scoped guard (compile-time !Send footgun protection); 7 duplex tests
- `tests/tcp_integration.rs` 6 real-TCP tests: round-trip, multi-client shared, oversize-frame teardown, partial-prefix EOF, pipelined order, concurrent INCR via Mutex
- `main.rs` 6 unit tests on `parse_args` grammar; lazy tokio runtime on server branch; ctrl_c → JoinSet drain

## Reading Companions
- **Rust:** The Rust Book (free, official). Read organically.
- **Domain:** DDIA (Kleppmann). One chapter per milestone (M3→ch.3, M4–5→ch.11, M6→ch.5, M7→ch.9, M8→ch.6).
- **Async (M2):** Tokio official tutorial.

## Key Paths
- `/home/netrom/kintoun` — project root
- `~/.claude/projects/-home-netrom-kintoun/memory/` — auto-memory (active)
- `Cargo.toml`, `src/{main,lib,cmd,storage,executor,repl,format,server}.rs` + `src/server/codec.rs`
- `docs/adr/0001-…0013-*.md` — ADRs (Nygard format), 13 total
- `docs/dev-setup.md` — local toolchain + bacon + IDE format-on-save + pre-commit hook
- `.github/workflows/ci.yml` — fmt --check + clippy -D warnings + cargo-llvm-cov + cargo-deny
- `bacon.toml`, `.githooks/pre-commit`, `deny.toml` — local + CI infrastructure

## Now
- **M2 closed.** End-to-end smoke confirmed (real TCP SET → OK; SIGINT → exit 0). Next: kick off M3 — read DDIA ch.3 (Storage and Retrieval), then design pass for WAL format + replay strategy.

## Open Threads
- **M3 design pass** — WAL (Write-Ahead Log) format, append/fsync policy, replay on startup, integration with `Storage` trait (likely a new `WalBackedStorage` impl that inherits the contract test suite).
- **ADR 0013 status note** — added 2026-05-06; flat-file module layout, `serve` signature, JoinSet drain, lazy runtime captured as deltas.
- **Stop hook prompt re-enable** — currently disabled. Revisit after deciding whether to retool the prompt or accept noise.
- **Coverage threshold ratchet** — at M3 once baseline stabilizes (ADR 0011).
- **Graceful drain timeout** — ADR 0013 follow-up; revisit at M5+ when long-running consumer connections appear.
- **Post-M1 quoting** — defer until something requires it (ADR 0008/0010 follow-ups).

## Recent Decisions
- 2026-05-06 (PM): **M2 closed.** `serve(listener, storage, shutdown: F)` with `JoinSet` drain (per-conn tasks tracked; on shutdown listener drops then `join_next` until empty; no drain timeout — ADR 0013 follow-up). 6 real-TCP integration tests (round-trip, multi-client, oversize, partial-prefix, pipelined, concurrent INCR). `main.rs` rewritten: hand-rolled `parse_args` (Mode = Repl | Server(SocketAddr)), 6 unit tests, lazy multi-thread tokio runtime on server branch only, `tokio::signal::ctrl_c()` wrapped to `Future<Output = ()>`. CLI: bare `--bind` errors with "missing argument" — explicit-value choice. End-to-end smoke: real TCP client SET → OK; SIGINT → exit 0. 129 tests project-wide.
- 2026-05-06 (AM): **M2 connection layer landed** (b14cf05). `handle_connection<IO, S>` generic over AsyncRead+AsyncWrite+Unpin and Storage+Send. `std::sync::Mutex` with block-scoped guard — compiler enforces no `MutexGuard` across `.await`. Error policy: codec/utf-8/write fatal → close; parse/executor → `ERR <msg>` continue. 7 duplex tests; `format_error` helper added to `format.rs`; `futures = "0.3"` dep added.
- 2026-05-04 (EOD): **M2 codec layer + format extraction landed.** ADRs 0012 (wire protocol, thicker Alternatives) + 0013 (server architecture) committed. `src/server/codec.rs` length-prefixed FrameCodec (13 tests + proptest); `src/format.rs` extracted from repl.rs (10 tests). Tokio + tokio-util + bytes added to Cargo.toml. `.claude/backups/` gitignored.
- 2026-05-04: **M2 design pass complete.** Six decisions locked: TCP+envelope wire format (B1: length-prefix + text payload), sync REPL + async server alongside, `Arc<Mutex<InMemoryStorage>>` sharing, `src/server.rs` flat-file layout (Rust 2018+ style) + `src/server/{codec,connection}.rs`, three-layer testing strategy, anyhow/thiserror boundary consistent with REPL.
- 2026-05-04: **M1 close.** Generic `Storage` contract test suite landed (per ADR 0006 point D); 22 contract fns + macro-generated wrappers. M3's WalBackedStorage will inherit by adding one wrapper module.
- 2026-05-04: First **proptest** properties on `from_text` — round-trip on success path, fall-through on failure path. Each runs ~256 cases per CI run.
- 2026-05-04: **cargo-llvm-cov** in CI replaces `cargo test` (single instrumented run, not double). No threshold yet — measure first, ratchet at M3. ADR 0011.
- 2026-05-04: Local **fmt parity** locked via three layers: bacon `fmt` job, `.githooks/pre-commit`, IDE format-on-save (multi-IDE docs in `docs/dev-setup.md`). Caught after CI fmt failure post-M1-polish commit.
- 2026-05-04: **REPL** generic over BufRead + Write + Storage; Redis-like format ("OK", "(nil)", quoted Str, bare Int/Counter, 1/0 for existence); anyhow at boundary, thiserror at layers (ADR 0010).
- 2026-05-04: **executor** layered design: storage strict (no coercion at apply), executor infers via `StoredValue::from_text` once on entry. Proven by keystone test `set foo "5"; incr foo → 8`. ADR 0008.
- 2026-05-04: **StoredValue** tagged enum (`Str`/`Int`); supersedes ADR 0005's `Option<&str>` and `value: String`. Read returns `Option<StoredValue>` (owned).
- 2026-05-04: **cargo-deny** in CI with permissive license allowlist + v2 advisory defaults; `unused-allowed-license = "allow"` to silence forward-looking-policy warnings. ADR 0009.
- 2026-05-02: Parser **case-insensitive** on verbs; trailing args **rejected** via `TooManyArgs`; quoted/multi-word values **deferred** post-M1; `ParseError` migrated to `#[derive(thiserror::Error)]`; ADR format = **Nygard**; GitNexus removed.
- 2026-05-01: `kintoun@0.0.1` published; dual-licensed MIT OR Apache-2.0.

## Blockers
- (none)
