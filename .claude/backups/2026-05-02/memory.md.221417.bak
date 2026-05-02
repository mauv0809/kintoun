# Memory

## Project
- **Name:** **kintoun** (crate + directory + GitHub repo). Directory: `/home/netrom/kintoun` (renamed from `/home/netrom/nimbus` on 2026-05-01). Future cloud version: `kintoun.cloud` — Dragon Ball callback (kintoun = 筋斗雲, Goku's Flying Nimbus, Japanese original; `nimbus` was taken on crates.io).
- **What:** Distributed KV store with stream/queue/cluster ambitions, built as the vehicle for learning Rust.
- **Language/build tool:** Rust + Cargo (edition 2024). Toolchain at `~/.cargo/bin` — Bash tool doesn't source `~/.cargo/env`; prepend to PATH or use full paths.
- **State (2026-05-01 EOD):** **`kintoun@0.0.1` shipped to crates.io** — name locked. Crate + directory + GitHub repo all aligned (`mauv0809/kintoun`, `/home/netrom/kintoun`). Dual-licensed MIT OR Apache-2.0. **`cmd.rs` parser at 7/7 green** — `set`, `get`, `incr` (default + with amount + invalid amount) arms + catch-all + empty-input handling all working. `ParseError` carries 4 variants tuned to data shape (Empty / MissingArg(&'static str) / UnknownCommand(String) / InvalidAmount { input, reason }).
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
- **Crate name:** `kintoun` (renamed from `nimbus` on 2026-05-01 — `nimbus` was taken on crates.io).
- **Testing strategy: Heavy + TDD discipline.** Red-green-refactor. ~35–50 tests across parser/storage/executor/repl + property tests. Storage trait tests double as the contract suite for M3+ impls. Dev-deps: `proptest`, optionally `pretty_assertions`. CI runs `cargo test` from first commit with tests.
- **Test-writing split:** Per layer — user writes first 2–3 tests by hand to learn Rust testing idioms; Claude expands the suite; user implements to green; we review user's implementation together. See auto-memory `feedback_test_writing_split.md`.

## Reading Companions
- **Rust:** The Rust Book (free, official). Read organically as topics surface.
- **Domain:** DDIA (Kleppmann). One chapter per milestone (M3→ch.3, M4–5→ch.11, M6→ch.5, M7→ch.9, M8→ch.6).
- **Async (M2):** Tokio official tutorial.

## Key Paths
- `/home/netrom/kintoun` — project root (renamed from `/home/netrom/nimbus` on 2026-05-01)
- Auto-memory will move to `~/.claude/projects/-home-netrom-kintoun/memory/` on next Claude session start. Old paths at `-home-netrom-nimbus/` and `-home-netrom-learn-rust/` are stale; can be deleted after migration is verified.
- `Cargo.toml`, `src/{main,lib,cmd,storage,executor,repl,error}.rs` — Layout 2, crate name `kintoun`
- `tests/kv_integration.rs` — optional integration tests
- `.github/workflows/ci.yml` — fmt --check + clippy -D warnings + test (live as of 043026)

## Now
- Day complete (2026-05-01). Crate published, parser landed with 7/7 tests, error model expanded organically with the implementation. Solid stopping point. Pick up tomorrow on one of: hand-roll `Display` + `std::error::Error` for `ParseError` (the educational pass per M1 design), backfill `del`/`exists`/`decr` arms with TDD cycles, or move to `storage.rs`.

## Open Threads
- **Hand-roll `Display` + `Error` for `ParseError`** — yesterday's M1 design called this the "educational pass" (see the trait once before reaching for `thiserror`). Currently `ParseError` only derives `Debug` + `PartialEq`. ~20 min, no tests change. **Recommended next** — locks in the trait literacy before more code stacks up.
- **Backfill `del`/`exists`/`decr` arms** — three near-identical TDD cycles (test → arm). ~15 min, locks full verb coverage.
- **Edge-case parser tests** — whitespace runs, casing (`SET` vs `set`), trailing args, multi-word values for `set`. May surface new `ParseError` variants or impl tweaks.
- **`storage.rs`** — next module per Layout 2. `Storage` trait + `InMemoryStorage` over `HashMap` + apply-mutation pattern (`Mutation` + `MutationOutcome`). New territory: traits with associated types, generics. ~30–60 min.
- Cleanup: `rm -rf ~/.claude/projects/-home-netrom-learn-rust` and `rm -rf ~/.claude/projects/-home-netrom-nimbus` once the new `-home-netrom-kintoun/` auto-memory path is verified working next session.

## Recent Decisions
- 2026-05-01: Renamed crate `nimbus` → `kintoun` (Goku's Flying Nimbus, Japanese original). Reason: `nimbus` taken on crates.io. Directory + GitHub repo already match (`mauv0809/kintoun`).
- 2026-05-01: Dual-licensed MIT OR Apache-2.0 (Rust ecosystem norm). `LICENSE-MIT` + `LICENSE-APACHE` files; `license = "MIT OR Apache-2.0"` in manifest.
- 2026-05-01: Decided to publish v0.0.1 to crates.io (`0.0.x` over `0.1.0` to honestly signal pre-alpha — name reservation only).
- 2026-05-01: Project directory renamed `/home/netrom/nimbus` → `/home/netrom/kintoun` to match crate + GitHub repo. Auto-memory path migrates next session.
- 2026-05-01: TDD cadence = one test at a time, full red→green cycle (preferred over batched Phase-1/Phase-2). User writes each test body; Claude reviews + provides arm scaffolds.
- 2026-05-01: `ParseError` expanded organically as needed: `MissingArg(&'static str)` for compile-time field labels (zero alloc), `UnknownCommand(String)` for user-input verb (dynamic), `InvalidAmount { input, reason }` as a struct variant carrying both the offending input AND the parse-error description.
- 2026-05-01: Asymmetric error variants for asymmetric data sources is a deliberate choice — `&'static str` when the data is a hardcoded label you control, `String` when it comes from user input, struct variant when you want both pieces of context.
- 2026-05-01: Manual `.map_err(|e| ...)` chosen over `From<ParseIntError> for ParseError` impl — `From` only sees the source error, can't reach back to the calling scope. `.map_err` closure captures `s` from outer scope, so the bad input gets echoed back in the error.
- 2026-05-01: Parser scope settled at 7/7 tests covering `set` / `get` / `incr` (default + with amount + invalid amount) / empty input / unknown verb. `del` / `exists` / `decr` arms intentionally still `todo!()`-equivalent (not yet implemented), per per-arm TDD discipline.
- 2026-04-30: Bootstrap complete — `Cargo.toml` deps `thiserror = "2"`, `anyhow = "1"`, dev `proptest = "1"` + `pretty_assertions = "1"`; CI workflow at `.github/workflows/ci.yml` (fmt --check + clippy -D warnings + test, w/ `Swatinem/rust-cache@v2`).
- 2026-04-30: `lib.rs` + `main.rs` split landed. `lib.rs` declares `pub mod cmd;`. `main.rs` stays hello-world until `repl.rs` exists, then becomes a thin shim calling `kintoun::repl::run(...)`.
- 2026-04-30: Struct variants over tuple variants for `Command` — names enforce field semantics; protects against k/v swap bugs and pays off when fields multiply.
- 2026-04-30: `Incr`/`Decr` use `by: u64` (rejected `i64`). Principle of least surprise — "incrementing by a negative" reads wrong; `s.parse::<u64>()` rejects negatives for free.
- 2026-04-30: Per-arm TDD discipline — `from_str` arms only get implemented when a red test demands them. `del`/`exists`/`decr` stay `todo!()` for now (no tests yet).
- 2026-04-30: M1 testing = Heavy + TDD discipline. Red-green-refactor; ~35–50 tests; `proptest` for properties; REPL `run()` generic over `BufRead`/`Write` for testability.
- 2026-04-30: M1 error model = per-module errors; `ParseError` hand-rolled, `StorageError` via `thiserror`, `main.rs` uses `anyhow`.
- 2026-04-30: Module Layout 2 with `lib.rs` + `main.rs` split, one concept per file.
- 2026-04-30: Storage Shape B — apply-mutation pattern with `Mutation` + `MutationOutcome`.
- 2026-04-30: M1 commands = `get`/`set`/`del`/`exists`/`incr`/`decr`; dispatch via `enum Command` + `match` exhaustiveness.
- 2026-04-30: Crate named **nimbus**; domain = distributed KV/streams/queues/cluster (Option B); persona/niche deferred to ~M4; Raft paper deferred to M7.

## Blockers
- (none)
