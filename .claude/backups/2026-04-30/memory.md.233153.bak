# Memory

## Project
- **Name:** **nimbus** (crate + directory). Future cloud version: `nimbus.cloud` — Dragon Ball callback to Goku's Flying Nimbus.
- **What:** Distributed KV store with stream/queue/cluster ambitions, built as the vehicle for learning Rust.
- **Language/build tool:** Rust + Cargo. Cargo project not yet initialized; will run `cargo init --name nimbus`.
- **State (2026-04-30):** Git repo initialized. Directory renamed to `~/nimbus`. Auto-memory migrated to `-home-netrom-nimbus` path.
- **Persona/niche:** Deferred. Decide ~M4 once friction surfaces real angles.

## Workflow Rules (load every session)
- **Pseudocode-first** + **substantial starter kits for Rust beginners.** User is brand-new to Rust syntax. For each new layer, post a rich starter kit in chat (imports, module structure, one worked example, skeleton TODOs, inline syntax notes). User adapts and re-types into their files. They write the meaningful logic; I provide the scaffolding that gets them off zero. See auto-memory `feedback_coaching_pseudocode_first.md` + `feedback_rust_beginner_scaffolding.md`.
- **Loop:** design together → user codes → review together. Don't race ahead.
- **Frame Rust against other languages** the user already knows.
- **Always explain acronyms on first use** (REPL, WAL, RPC, etc.) until told to stop. See auto-memory `feedback_explain_acronyms.md`.
- **Hold the line on milestone scope.** Don't pre-design future milestones; only forward-looking constraint is "don't make M1 decisions that block M4–M8."

## Milestone Arc (locked 2026-04-30)
- M1: In-memory KV + REPL (single binary, no network) ← **active**
- M2: TCP server + framed protocol (tokio async)
- M3: WAL persistence + replay
- M4: Pub/sub event streaming on the log
- M5: Consumer groups + offsets (queue semantics)
- M6: Single-leader async replication
- M7: Raft-lite leader election (read Raft paper here, not before)
- M8: Partitioning/sharding
- Stretches: snapshots, transactions, gRPC, metrics, client SDK

## M1 Design (LOCKED 2026-04-30)
- **Commands:** `get`, `set`, `del`, `exists`, `incr`, `decr`.
- **Dispatch:** `enum Command` + `match` exhaustiveness. Parser: `impl FromStr for Command` using `split_whitespace`.
- **Storage shape:** apply-mutation pattern — `Mutation` enum + `MutationOutcome` enum + `Storage::apply()`. Reads (`read`, `exists`) bypass `Mutation`. M1 impl: `InMemoryStorage` over `HashMap`.
- **Error model:**
  - Per-module errors (no single global error).
  - `cmd::ParseError` — **hand-rolled** `impl Display` + `impl std::error::Error` (the educational pass — see the trait once).
  - `storage::StorageError` — `#[derive(thiserror::Error)]`.
  - `main.rs` returns `Result<(), anyhow::Error>` for ergonomic top-level. `Cargo.toml` adds: `thiserror`, `anyhow`.
- **Module layout (Layout 2):** `src/{main,lib,cmd,storage,executor,repl,error}.rs` + optional `tests/kv_integration.rs`. `lib.rs` + `main.rs` split.
- **REPL input:** plain `std::io::stdin().read_line()` in a loop. No deps. Trivial swap to `rustyline` post-M1 if desired. **REPL `run()` generic over `BufRead` + `Write`** for testability.
- **Crate name:** `nimbus`.
- **Testing strategy: Heavy + TDD discipline.** Red-green-refactor. ~35–50 tests across parser/storage/executor/repl + property tests. Storage trait tests double as the contract suite for M3+ impls. Dev-deps: `proptest`, optionally `pretty_assertions`. CI runs `cargo test` from first commit with tests.
- **Test-writing split:** Per layer — user writes first 2–3 tests by hand to learn Rust testing idioms; Claude expands the suite; user implements to green; we review user's implementation together. See auto-memory `feedback_test_writing_split.md`.

## Reading Companions
- **Rust:** The Rust Book (free, official). Read organically as topics surface.
- **Domain:** DDIA (Kleppmann). One chapter per milestone (M3→ch.3, M4–5→ch.11, M6→ch.5, M7→ch.9, M8→ch.6).
- **Async (M2):** Tokio official tutorial.

## Key Paths
- `/home/netrom/nimbus` — project root
- Auto-memory: `~/.claude/projects/-home-netrom-nimbus/memory/`
- Old auto-memory at `~/.claude/projects/-home-netrom-learn-rust/memory/` is stale; can be deleted.
- `Cargo.toml`, `src/{main,lib,cmd,storage,executor,repl,error}.rs` — Layout 2, after `cargo init --name nimbus`
- `tests/kv_integration.rs` — optional integration tests
- `.github/workflows/` — TBD post-bootstrap

## Now
- M1 design **complete**. Ready to bootstrap (`cargo init`, etc.) or to begin user-driven implementation.

## Open Threads
- **Bootstrap (next):** `cargo init --name nimbus`, write `.gitignore`, first commit of Claudify scaffolding + cargo skeleton, decide rustfmt/clippy config, GitHub Actions CI stub.
- **First implementation slice (TDD):** Start at `cmd.rs`. Write 1 failing test (e.g., `parse("") → Err(ParseError::Empty)`), then minimum code to pass, then add the next test. Review the layer with me when ~5–8 tests are green.
- Cleanup: `rm -rf ~/.claude/projects/-home-netrom-learn-rust` once new path is verified working.
- Recommended: restart Claude in `~/nimbus` for clean cwd before bootstrap.

## Recent Decisions
- 2026-04-30: M1 testing = Heavy + TDD discipline. Red-green-refactor; ~35–50 tests; `proptest` for properties; REPL `run()` generic over `BufRead`/`Write` for testability.
- 2026-04-30: M1 error model = per-module errors; `ParseError` hand-rolled, `StorageError` via `thiserror`, `main.rs` uses `anyhow`.
- 2026-04-30: M1 REPL input = plain `stdin().read_line()`, no crate.
- 2026-04-30: Crate named **nimbus**; directory renamed to `~/nimbus`. Auto-memory migrated.
- 2026-04-30: Module Layout 2 with `lib.rs` + `main.rs` split, one concept per file.
- 2026-04-30: Storage Shape B — apply-mutation pattern with `Mutation` + `MutationOutcome`.
- 2026-04-30: Dispatch via `enum Command` + `match` exhaustiveness.
- 2026-04-30: M1 commands = `get`/`set`/`del`/`exists`/`incr`/`decr`.
- 2026-04-30: Domain locked = distributed KV/streams/queues/cluster (Option B).
- 2026-04-30: Defer persona/niche decision until ~M4.
- 2026-04-30: Skip Raft paper until M7. Read DDIA + Rust Book organically.

## Blockers
- (none)
