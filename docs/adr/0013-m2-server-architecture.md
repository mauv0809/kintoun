# ADR 0013: M2 Server Architecture — Async Server Alongside Sync REPL with Arc<Mutex> Storage

Date: 2026-05-04
Status: Accepted (implemented 2026-05-06; see Status Notes)

## Context

ADR 0012 locks the M2 wire format. This ADR locks the runtime architecture: where the server code lives, how concurrent connections share state, how the existing sync REPL and the new async server coexist, and how the whole thing is tested.

The forcing constraints:

- **Tokio's async I/O traits are different from the sync ones.** `AsyncBufRead`/`AsyncWrite` are not `BufRead`/`Write`. Rust has no clean abstraction over both — the "function color" problem. M1's `repl::run` is generic over the sync traits and works fine for an interactive shell.
- **Async-ifying the REPL would buy nothing.** One client, no concurrency to exploit; pays the function-color tax with no upside.
- **Many concurrent TCP clients need shared access to `InMemoryStorage`.** Safety has to be guaranteed by the borrow checker, not by convention.

These choices are mostly mechanical. ADR 0012 carries the high-stakes weight; this ADR captures the structural decisions for posterity and for anyone (future-us included) reading the code without the conversation context.

### Alternatives Considered

**Async-ify the REPL alongside the server.** Use tokio's async stdin/stdout. Rejected: gratuitous async machinery for a single-client interactive shell. Async earns its keep where concurrency exists; the REPL has none.

**`Arc<RwLock<InMemoryStorage>>` for storage sharing.** Defensible — parallel reads matter at scale. Rejected for M2: critical sections are O(1) hashmap ops measured in microseconds; contention is theoretical at small scale; deadlock surface is richer than `Mutex`; the `Storage` trait abstracts the lock so a future swap doesn't touch call sites. RwLock as a *learning topic* is worth knowing exists; not worth picking for M2 unless the lesson is the goal.

**`Arc<DashMap<String, StoredValue>>` (lock-free sharded concurrent map).** Rejected: third-party dependency; loses the `Storage` trait abstraction unless wrapped (which negates much of the benefit); harder to add atomic multi-key operations at M3+ (batch writes, transactions).

**Actor pattern (one task owns storage, others send messages via `mpsc` + `oneshot` channels).** Rejected: more machinery; per-request channel allocation cost; effectively serializes through one task anyway, so functionally equivalent to `Mutex` but with extra steps. Worth knowing as a pattern; not the right tool here.

**Flat `src/server.rs` (one file).** Rejected: M4 push frames and M6 peer protocol will grow the module; starting nested costs nothing now and avoids a Rust module reorg later (path renames, `mod` declarations).

**`clap` or another CLI parser dependency.** Rejected for M2: one optional flag (`--bind`) does not justify a dependency. Revisit at three or more flags.

**`--port` only flag (no address override).** Rejected: locks kintoun to localhost permanently; M6 will require non-localhost binding for cross-host replication. `--bind <addr:port>` is the same hand-rolled parsing effort and keeps the door open.

## Decision

1. **Sync REPL stays; async server lives alongside.** `src/repl.rs` is unchanged. `cargo run` continues to open the interactive shell. The async server is invoked as a separate entry point. (Whether `cargo run` with no args opens the REPL or the server, and how the user picks between them, is a small implementation-time question — not load-bearing for this ADR.)

2. **Module layout: `src/server/{mod,codec,connection}.rs`.**
   - `mod.rs` — public `run` function; listener loop; binding; shutdown wiring.
   - `codec.rs` — frame encoder/decoder (the envelope from ADR 0012).
   - `connection.rs` — per-connection async task body; reads frames, dispatches to `executor`, writes frames back.

3. **Concurrency: one tokio task per connection.** Tasks are cheap (a few KB each, no OS thread); one runtime thread can host thousands. Storage is shared via `Arc<Mutex<InMemoryStorage>>`. Standard library `Mutex` (or `parking_lot::Mutex`) is fine — our storage operations are sync and hold the lock only for microseconds. The lock is never held across an `.await` point.

4. **Lifecycle.**
   - Default bind: `127.0.0.1:4242`.
   - Override: `--bind <addr:port>` flag, hand-rolled parsing in `main.rs`. No `clap` dependency.
   - Shutdown: `tokio::signal::ctrl_c()` triggers graceful stop. Drop the listener; let in-flight connections finish naturally as their clients close; exit. No drain timeout at M2.
   - No max-connection cap. Tokio handles many; cap only if resource exhaustion becomes observable.

5. **Testing — three layers.**
   - **Codec unit tests.** Pure sync. Encode/decode round-trips; partial-frame handling (insufficient bytes); frame-glued buffer handling (decode returns first frame plus leftover bytes). The codec is the genuinely new lesson; bugs here cascade.
   - **In-memory connection tests.** `#[tokio::test]` with `tokio::io::duplex(N)` — an in-memory bidirectional pipe with an N-byte buffer — fakes a TCP connection. Drives the connection handler with a "client side" of the duplex. Verifies handler ↔ codec ↔ executor ↔ storage wiring without real network.
   - **End-to-end TCP integration test.** `tests/tcp_integration.rs`. Bind a real `TcpListener` on `127.0.0.1:0` (port 0 = OS-picked free port); read the bound port back; spin up the server; connect a real client; round-trip a few commands. Covers OS plumbing — the part the in-memory tests can't reach.

6. **Errors.** `anyhow` at the server boundary (top-level connection task and listener task); `thiserror` at typed layers (codec errors, I/O errors). Errors sent to client as `ERR <message>` frames per ADR 0012. Boundary convention is consistent with REPL per ADR 0010 — this is the project-wide pattern now, not a per-binary choice.

## Consequences

- **REPL stays simple.** Async machinery is confined to where it earns its keep. `src/repl.rs` is the same file it was at M1; no diff from this ADR.
- **The layered design from M1 is reused unchanged.** `cmd`, `executor`, and `storage` are I/O-agnostic; only the I/O frame differs between REPL and server. This validates the layered design from ADR 0003 / 0008.
- **`Arc<Mutex>` is correct-by-construction.** The borrow checker enforces safety. Lock-free designs require more care; we use them when they pay for themselves.
- **The `Storage` trait abstracts the lock.** Swapping to `RwLock` or sharded storage at a later milestone touches the binding code, not call sites. The cost of "wrong choice now" is bounded.
- **Three-layer testing catches different bug classes.** Codec correctness (unit), handler wiring (in-memory), OS plumbing (integration). Each layer is small; together they cover the M2 surface.
- **Two parallel I/O surfaces (REPL + server).** Doubled surface area for I/O bugs in principle. Mitigated by the shared lower layers — the I/O code itself is small relative to the logic.
- **`Arc<Mutex>` serializes all storage access.** A future high-contention workload would need swapping. Mitigated: deferred until benchmarks show it matters; trait abstraction makes the swap mechanical.
- **The function-color split is permanent.** Sync code in `repl.rs` and async code in `server/` cannot share I/O helpers. Acceptable cost for the simpler structure.
- **No dependency added for CLI parsing.** One flag is hand-rolled; the project keeps a lean dependency tree until pressure justifies otherwise.

## Open Follow-ups

- **Default mode of `cargo run`.** REPL by default with a `--server` flag, or server by default with a `--repl` flag, or two binaries (`kintoun-shell`, `kintoun-server`). Decide at implementation time; not load-bearing for the ADR.
- **Frame-size cap value (per ADR 0012).** Default 16MB; revisit on real load.
- **`RwLock` or sharded storage.** Add when benchmarks show contention. The `Storage` trait makes the swap mechanical.
- **Graceful drain timeout.** M2 ships with drop-and-let-finish. If shutdown latency becomes an issue (e.g., long-running consumer connections at M5), add a configurable drain deadline.
- **Max concurrent connections cap.** M2 has none. Add only if resource exhaustion is observed; bound the cap by available file descriptors and memory.
- **Configurable `KINTOUN_BIND` environment variable.** In addition to the `--bind` flag. Cosmetic; defer.

## Status Notes (2026-05-06)

Minor deviations from the original Decision section, captured at implementation time:

- **Module layout uses Rust 2018+ flat-file form.** The ADR specified `src/server/{mod,codec,connection}.rs`; the implementation uses `src/server.rs` (flat file holding `pub mod codec; pub mod connection;` and the `serve` function) plus the `src/server/{codec,connection}.rs` submodules. Equivalent module shape; idiomatic Rust 2018+. No semantic change.

- **Public function is `serve(listener, storage, shutdown)`, not `run`.** Caller owns the bind (passes a `TcpListener`) and the shutdown signal (passes a `Future<Output = ()>`). This shape lets tests pass `std::future::pending()` for "serve forever" and the binary pass `tokio::signal::ctrl_c()`. Tighter separation of concerns than the ADR sketched.

- **Graceful shutdown uses `tokio::task::JoinSet`.** Per-connection tasks are tracked in a `JoinSet`; on shutdown the listener drops and the JoinSet is drained via `join_next` until empty. This is a stricter implementation than the ADR's "let in-flight finish naturally" — the function actually awaits drain rather than relying on runtime shutdown to do it. Drain has no timeout (Open Follow-up: "Graceful drain timeout").

- **`--bind <addr>` requires a value.** No bare-`--bind` shorthand for the default 127.0.0.1:4242. Most explicit; bare `--bind` errors with "missing argument."

- **Lazy tokio runtime.** `fn main` is sync; the runtime is built only on the server branch. REPL stays runtime-free, matching the principle "async earns its keep where concurrency exists" from the ADR's Context section.
