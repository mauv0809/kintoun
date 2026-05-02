# Memory

## Project
- **Name:** **kintoun** (crate + directory + GitHub repo). Directory: `/home/netrom/kintoun`. `kintoun = 筋斗雲`, Goku's Flying Nimbus (Japanese original); `nimbus` was taken on crates.io. Future cloud version: `kintoun.cloud`.
- **What:** Distributed KV store with stream/queue/cluster ambitions, built as the vehicle for learning Rust.
- **Language/build tool:** Rust 2024, Cargo. MSRV 1.85. Toolchain at `~/.cargo/bin` — Bash tool doesn't source `~/.cargo/env`; prepend to PATH or use full paths.
- **State (2026-05-02 EOD):** Parser feature-complete (29 tests, full M1 grammar). 7 ADRs in `docs/adr/`. README has Architecture section. `rustfmt.toml` + `clippy.toml` pinned. GitNexus removed. **`storage.rs` at stopping point A** — design surface (Mutation / MutationOutcome / StorageError / Storage trait / InMemoryStorage skeleton); no `impl Storage for InMemoryStorage` block yet. Stop hook prompt step disabled (model wasn't honoring JSON-only directive); logger script preserved.
- **Persona/niche:** Deferred. Decide ~M4 once friction surfaces real angles.

## Workflow Rules (load every session)
- **Pseudocode-first** + **substantial starter kits for Rust beginners.** User adapts and re-types into their files. They write the meaningful logic; I provide scaffolding.
- **Loop:** design together → user codes → review together. Don't race ahead.
- **Frame Rust against other languages** the user already knows.
- **Always explain acronyms on first use** (REPL, WAL, RPC, etc.) until told to stop.
- **Hold the line on milestone scope.** Only forward-looking constraint: "don't make M1 decisions that block M4–M8."
- **Push back when substance warrants.** Concede only when the user's argument actually moves a premise; don't capitulate to social pressure. See feedback memories.

## Milestone Arc (locked 2026-04-30)
- M1: In-memory KV + REPL ← **active**
- M2: TCP server + framed protocol (tokio)
- M3: WAL persistence + replay
- M4: Pub/sub event streaming on the log
- M5: Consumer groups + offsets (queue semantics)
- M6: Single-leader async replication
- M7: Raft-lite leader election
- M8: Partitioning/sharding

## M1 Module Status
- `cmd.rs` — ✅ feature-complete, 29 tests
- `storage.rs` — ⏳ point A done; impl block (point B) is next
- `executor.rs` — ❌ not started (turns Commands into Storage ops)
- `repl.rs` — ❌ not started (generic over `BufRead` + `Write`)
- `main.rs` — ⏳ still hello-world; becomes thin shim once REPL lands
- `tests/kv_integration.rs` — ❌ optional; lands when layers connect

## Reading Companions
- **Rust:** The Rust Book (free, official). Read organically.
- **Domain:** DDIA (Kleppmann). One chapter per milestone (M3→ch.3, M4–5→ch.11, M6→ch.5, M7→ch.9, M8→ch.6).
- **Async (M2):** Tokio official tutorial.

## Key Paths
- `/home/netrom/kintoun` — project root
- `~/.claude/projects/-home-netrom-kintoun/memory/` — auto-memory (active)
- `Cargo.toml`, `src/{main,lib,cmd,storage,...}.rs` — Layout 2
- `docs/adr/0001-…0007-*.md` — ADRs (Nygard format)
- `.github/workflows/ci.yml` — fmt --check + clippy -D warnings + test

## Now
- Day complete (2026-05-02). Parser shipped, ADRs landed, configs pinned, GitNexus removed, storage.rs design surface in place. Tomorrow: stopping point B (first `impl Storage for InMemoryStorage` block, `Set` mutation + `Get` read, 2-4 unit tests).

## Open Threads
- **`storage.rs` point B (next)** — first impl Storage block; Set + Get; ~30-45 min. Stale `#[expect(dead_code)]` on `data` field self-cleans when impls touch it.
- **`storage.rs` point C** — all four mutations + both reads; locks 5 open behavior decisions (incr-missing, incr-non-numeric, decr-underflow, del-missing, exists semantics) via TDD. ~60-90 min from B.
- **`storage.rs` point D** — generic Storage contract test suite (per ADR 0006). Harder Rust (generic test functions). ~30-60 min beyond C.
- **`executor.rs`, `repl.rs`, wire `main.rs`** — remaining M1 modules.
- **Stop hook prompt re-enable** — currently disabled. Revisit after deciding whether to retool the prompt or accept noise.
- **Cleanup leftovers** — `rm -rf ~/.claude/projects/-home-netrom-{nimbus,learn-rust}` (verified migrated); `rm -rf .claude/skills/gitnexus`; `rm -rf ~/.claude/hooks/gitnexus`.

## Recent Decisions
- 2026-05-02: Parser is **case-insensitive** on verbs (`to_ascii_lowercase()`); keys/values stay case-sensitive.
- 2026-05-02: Trailing args **rejected** via `ParseError::TooManyArgs(&'static str)` (tuple variant per rubric); `expect_done` helper.
- 2026-05-02: Quoted/multi-word values **deferred** post-M1.
- 2026-05-02: `ParseError` Display+Error migrated hand-roll → `#[derive(thiserror::Error)]` with `#[error("...")]` attributes (educational pass complete).
- 2026-05-02: Original-case verb preserved in `UnknownCommand` errors; tested.
- 2026-05-02: ADR format = **Nygard**. 7 ADRs landed (0001 toolchain, 0002 crate name, 0003 module layout, 0004 error model, 0005 storage shape, 0006 TDD, 0007 grammar).
- 2026-05-02: README has Architecture section pointing at ADR 0005.
- 2026-05-02: `Storage::read` returns **`Option<String>`** (owned). Refactor to owned was certain by M6+; absorbed cost upfront.
- 2026-05-02: `Storage` trait uses **concrete types** (not associated types). Refactor to associated isn't certain.
- 2026-05-02: GitNexus removed (low ROI at M1 single-module scale; revisit at M2+).
- 2026-05-02: Stop hook prompt step disabled — neither sonnet nor haiku honored JSON-only directive reliably; logger script preserved.
- 2026-05-01: `kintoun@0.0.1` published; crate name + dir + repo aligned; dual-licensed MIT OR Apache-2.0.
- 2026-04-30→05-01: M1 design locked; bootstrap complete; parser landed via per-arm TDD. Captured in ADRs 0001-0007.

## Blockers
- (none)
