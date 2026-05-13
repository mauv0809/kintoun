# ADR 0014: M3 WAL — Effects Log, Framed Binary Records, Single-Impl Storage with Optional WAL

Date: 2026-05-07
Status: Accepted (implementation pending — M3 Phase 1)

## Context

M2 closed with kintoun durable only for the lifetime of a single process — every SET/INCR/DEL lives in `InMemoryStorage`'s HashMap and dies with it. M3 adds persistence: a Write-Ahead Log (WAL) that captures every state mutation before it lands in memory, so state survives restart.

The WAL is the foundational durability primitive for the rest of the milestone arc:

- M3: persistence + replay (this ADR)
- M4: pub/sub event streaming — the log is the substrate
- M5: single-leader async replication — leaders ship effects from their WAL
- M6: replicated WALs; compaction; retention
- M7: Raft-lite — the WAL becomes the replicated log
- M8: partitioning — per-shard WALs

Format and policy decisions made here propagate. Per the project rule that ADR rigor scales with stakes, this ADR carries thicker Alternatives sections on the high-stakes decisions (#1 log content, #3 storage shape, #6 corruption handling) and tighter treatment on the lower-stakes ones (#2 record format, #4 fsync policy, #5 file layout, #7 concurrency).

Three pressures shaped the pass:

1. **Correctness at boundary.** A WAL is only a WAL if "the call returned" implies "on disk." Per-write fsync semantics, torn-write detection, and replay correctness must hold without hand-waving.
2. **Forward-pressure across the arc.** The M3 record format must survive M4 (consumed by stream subscribers), M5 (shipped to followers), M6 (compacted), and M7 (replicated by Raft) without a wire-format break.
3. **Industry-standard fit.** Production WALs (PostgreSQL, RocksDB, etcd, Redis AOF) integrate durability into the storage type, not decorate it. The trait-decorator-pattern-as-WAL temptation in OO/Rust trait designs is appealing aesthetically but architecturally wrong — durability is not a cross-cutting concern.

## Decisions

Seven decisions, ordered so each informs the next.

### #1 — Log content: effects (physical), not commands

**Decision.** Each WAL record encodes a resolved state change — `PUT key value` or `DELETE key` — not the original command line. Replay applies effects directly to a fresh in-memory map.

**Alternatives considered.**

**A — Commands (logical log).** Each record is the original command (`SET foo bar`, `INCR counter 5`). Replay re-runs commands from empty state through the existing parser + executor. Smallest M3 implementation (record format = the command grammar); `tail -f wal.log` reads like the user's session; reuses runtime code paths.

Rejected. Forces a refactor at M4: when the log becomes the stream substrate, subscribers want to consume *what changed* (state mutations), not *what was asked*. Command-logging would also undermine M5 replication — non-deterministic ops (theoretical at M3 but real once timestamps, randomness, or environmental dependencies enter) replay differently across nodes. Compaction at M6+ becomes hard: subsuming a sequence of `INCR` records into a final state requires simulating execution. Per the project rule against making decisions where a refactor is certain, command-logging fails the test on multiple axes.

**B — Effects (physical log, chosen).** Each record is the post-state mutation. Replay is "apply this PUT/DELETE to the map." Idempotent regardless of starting state. M4 stream subscribers consume effects naturally. M5 replication ships effects so replicas never re-execute. Determinism survives executor evolution: if INCR's bound-check semantics ever change, command-logged history would replay with new behavior; effect-logged history reflects the historical truth. Compaction is trivial — last-write-wins per key. Industry default per DDIA ch.3 (Bitcask, LSM-trees, most real DBs).

**C — Hybrid / event-sourced.** Log commands AND derive effects on the fly. Considered briefly; rejected as complexity without clear benefit at M3.

### #2 — Record format: framed binary header + UTF-8 text payload

**Decision.** Each WAL record is laid out as:

```
[total_len:u32 BE][seq:u64 BE][crc32:u32 BE][effect_text:utf8]
```

A 16-byte binary header followed by a UTF-8 effect line (`PUT key value` or `DELETE key`). `total_len` covers the payload bytes only (header is fixed-size and known). Sequence numbers are monotonically assigned at write time. CRC32 is computed over `total_len` + `seq` + `effect_text` — the full record minus the CRC field itself — using `crc32fast`. This protects every byte that participates in the record's semantics, not just the payload; a flipped bit in `total_len` or `seq` is caught the same way as a flipped bit in the value.

**Alternatives considered.**

**Pure line-delimited text.** One effect per line. Trivial parser; greppable. Rejected: no corruption detection beyond parse failure, and binary-clean values would force a quoting/escape rule that ADRs 0008/0010 explicitly deferred.

**Length-prefixed text payload (no CRC, no seq#).** Detects torn writes via length mismatch but adds binary framing without the bit-flip detection that usually justifies it. Strange middle ground; rejected.

**Pure binary payload.** Define a binary effect encoding now. Rejected as premature: no performance pressure at M3, and we'd be designing a binary key/value format for no reason. Text payload preserves debuggability (`xxd` over the 16-byte header, then a plain UTF-8 line).

The chosen format detects:

- **Torn writes** at record boundaries (length says N, EOF before N bytes).
- **Bit-flips and silent corruption anywhere in the record** (CRC32 over `total_len` + `seq` + payload — the full record minus the CRC field itself).
- **Sequence regressions or gaps** (monotonic seq# verified at replay).

Sequence numbers are M3 sanity checks; they become load-bearing at M5 for replication ordering and at M7 for Raft log indexing. Adding them now absorbs that future work.

### #3 — Storage shape: single-impl with optional WAL

**Decision.** One concrete impl of the `Storage` trait, `InMemoryStorage`, owns the HashMap and an optional WAL writer:

```rust
struct InMemoryStorage {
    map: HashMap<String, StoredValue>,
    wal: Option<WalWriter>,
}
```

The struct name remains `InMemoryStorage` — the data structure is still a HashMap in memory; the WAL is a durability companion, not a substrate change. This mirrors Redis (`dict` + AOF), where the dict is still the in-memory store regardless of whether AOF is enabled. Two constructors:

- `InMemoryStorage::new() -> Self` — no-WAL mode (in-memory only; preserves the M1/M2 entry point).
- `InMemoryStorage::open(path: impl AsRef<Path>) -> Result<Self>` — WAL-backed mode; opens the WAL, runs replay, and returns a fully-hydrated Storage ready for appends.

Mutations write to the WAL first (if present), then apply to the map. The `Storage` trait is kept for mockability and for the existing contract test suite (per ADR 0006 point D); the suite runs in two configurations against the single impl — once with `wal = None` (in-memory mode), once with `wal = Some(...)` (durable mode). Behavioral equivalence is proved by both configurations passing the same 22 contract tests.

**Alternatives considered (thicker — this was the most contested decision of the pass).**

**A — Reimplementation: a separate `WalBackedStorage` type alongside `InMemoryStorage`.** Both implement the `Storage` trait; the contract test suite runs against both. ~150 new lines of HashMap-using code. Drift risk between the two impls is caught by the contract suite.

Rejected. Medium industry fit — closest analog is a Java/Scala "DurableHashMap" wrapper class, but production WALs (PG, RocksDB, etcd, Redis) do not have two parallel storage impls. The "log first, apply second" invariant for INCR/DECR forces the second impl to duplicate inference logic from the first; the decorator-style "delegate to inner" path doesn't work for those operations because the effect depends on current state, which forces the wrapper to read-then-compute-then-log-then-apply manually. M5 also forces a refactor: replicas need an `apply(Effect)` entry point, which would be added to the trait + both impls.

**B — Trait redesign + decorator at `apply()`.** Decompose `Storage` into primitives: `preview_set`, `preview_incr`, `preview_delete`, and a single `apply(effect)`. High-level methods (`set`, `incr_by`, etc.) become default impls calling preview + apply. A `WalBackedStorage` decorator overrides only `apply()` to log first, then delegates. ~30 new lines + ~80 changed lines in the existing impl. The cleanest "single source of truth" — durability lives at exactly one method.

Rejected. **Worst industry-standard fit of the three options.** Production WALs are not decorators. Cross-cutting concerns (caching, metrics, retries, logging) are orthogonal to data semantics — they can layer on top of any storage without changing its contract. Durability is not in that bucket: when a write commits, what survives a crash, the order of log + state — these are storage-level invariants, not separable aspects. PostgreSQL fuses WAL into the buffer manager. RocksDB has WriteBatch fused into the write path. etcd's apply path goes raft → MVCC store + WAL as a pipeline. Kafka makes the log itself the storage. Redis hooks AOF into the command processor. None decorate.

The trait-decorator pattern shows up in Rust/Java/Scala for genuine cross-cutting concerns — but applying it to durability is mistaking a semantic invariant for an aspect.

**C — Single-impl with optional WAL (chosen).** One storage type, durability layered into its write path. Closest fit to Redis (`dict` + AOF integrated into command processor), PostgreSQL (buffer manager + WAL), RocksDB (one storage type, WriteBatch flow). M5 replication adds an `apply(Effect)` method to the single type — small, contained refactor (~15 lines). The trait stays for mockability; the contract suite runs in two configs, preserving the M1 investment in trait + contract.

The pivotal reframe during the design pass: the original recommendation was option B (cleanest Rust learning surface, smallest M5 refactor cost). User pushed back — *"How do we get closer to industry standard 'integrating WAL into the write path'?"* — and once industry-standard fit was weighted explicitly alongside the other axes, option C dominated. The recommendation shifted in mid-discussion. The lesson is recorded as a knowledge nomination: industry-standard fit is a load-bearing design axis for this project, not aesthetic.

**D — Single-impl, drop the trait entirely.** A consequence variant of C: with one impl, the trait is mostly vestigial. Considered and rejected: the trait still earns its keep for mockability in tests, and the two-config contract test pattern preserves drift detection. Drop the trait only if a future milestone makes it actively harmful.

### #4 — Append + fsync policy: per-write `sync_data()`

**Decision.** Every WAL append writes the record bytes to the file, calls `File::sync_data()` (`fdatasync(2)`), and returns to the caller only after sync completes. No batching, no time-bounded fsync, no background flush task.

**Alternatives considered (thinner).**

**Group commit / batched fsync.** Buffer multiple writes, fsync once per batch (timer or count threshold). Production throughput pattern (PostgreSQL `commit_delay`, MySQL group commit). ~150 lines of async coordination + commit-barrier logic. Rejected for M3 as premature: no throughput pressure at this milestone; the pattern is a pure optimization and can be added later without changing the on-disk format.

**Time-bounded (every N ms, async background flush).** Up to N ms of data loss on crash — strictly less safe than per-write fsync. Closest to Redis `appendfsync everysec`. Rejected: no operational reason to weaken durability at this milestone.

**No-fsync (rely on OS page cache flush).** Up to ~30s data loss on crash. Not a real option for anything called a WAL.

**Configurable (per-write / time-bounded / off via setting).** Production-shaped flexibility. Rejected for M3: three policies × testing matrix is over-engineered for the milestone.

`sync_data()` (`fdatasync`) is preferred over `sync_all()` (`fsync`) because we don't care about file metadata flushes (mtime); only data + new file size matter. Same convention as PG/MySQL/RocksDB.

### #5 — File layout: single growing file

**Decision.** One file `wal.log` that grows on every append. No segmentation at M3.

**Alternatives considered (thinner).**

**Segmented from M3.** Files like `wal.0001.log`, `wal.0002.log`, ... rotated when a size threshold is hit. Production shape across Kafka, etcd, RocksDB. ~70 extra lines for rotation, naming, segment registry, segment-aware writer.

Rejected as premature. Segments earn their keep when compaction (drop subsumed records) or retention (drop oldest data) become first-class concerns — i.e. M6+. M3 replay reads one file sequentially; M4 stream subscribers can consume from one file via byte-tail or seq-position cursor; M5 replication ships byte tails. Nothing pre-M6 forces segmentation.

The escape-hatch insight: per-record sequence numbers (decision #2) make records self-describing, so a single growing file is the degenerate "one segment" case. Switching to segments at M6 is a *file management* refactor, not a *record format* refactor — the existing `wal.log` becomes `wal.0001.log` with a rename, and existing record bytes are unchanged. Migration cost is small; deferral is cheap.

### #6 — Replay strategy and corruption handling

**Decision.** A constructor `Storage::open(path: impl AsRef<Path>) -> Result<Storage>` performs replay during construction and returns a fully-hydrated Storage with the WAL writer ready for appends. No separate "did you remember to call replay?" step.

Replay reads records sequentially. For each record:

1. Read the 16-byte header. If fewer than 16 bytes available (trailing torn header) → end of log; truncate file at this position; continue with what's been replayed.
2. Read `total_len` payload bytes. If fewer than `total_len` bytes available (trailing torn payload) → end of log; truncate; continue.
3. Verify CRC32 over `total_len` + `seq` + payload. On mismatch, classify trailing vs mid-log via one-record lookahead at the next record's expected offset (`current + 16 + total_len`):
   - **Lookahead reads < 16 bytes** (EOF or torn header) → trailing torn write; truncate file at the start of the bad record; warn.
   - **Lookahead reads a 16-byte header but the declared payload is short** (torn payload) → trailing; truncate; warn.
   - **Lookahead successfully reads a full subsequent record** (regardless of whether its own CRC validates) → mid-log corruption; refuse to start. Print "WAL corruption at record N (offset X, seq Y)" and exit non-zero.

   The replay loop does *not* selectively skip the bad record and apply the records after it. Skip-and-continue is the rejected alternative below; it produces silent data loss.
4. Verify monotonic seq# against the previous record (or 0 for the first record).
   - Gap or regression → refuse to start. Print "WAL ordering violation at record N (offset X, expected seq ≥ Y+1, got Z)" and exit non-zero.
5. Parse the effect text and apply to the in-memory map.

After replay, open the WAL writer in append mode at the (possibly truncated) end of the file.

**Alternatives considered (thicker — this was the second highest-stakes decision).**

The two real questions: how to handle **trailing** anomalies, and how to handle **mid-log** anomalies. Industry consensus across PG, RocksDB, etcd, and Redis AOF is the asymmetric pattern locked above:

| Scenario | Handling | Reasoning |
|---|---|---|
| Trailing torn header (< 16 bytes at EOF) | Truncate-and-warn | Unclean shutdown is expected; client never got ACK |
| Trailing torn payload | Truncate-and-warn | Same |
| Trailing CRC mismatch | Truncate-and-warn | Likely partial-write race during fsync |
| Mid-log CRC mismatch | Refuse-to-start | Real corruption — bit-flip, misdirected write |
| Mid-log seq gap or regression | Refuse-to-start | Ordering invariant is load-bearing |

The asymmetry reflects asymmetric *signal strength*: a torn-trailing record is a high-prior-probability outcome of any unclean shutdown; a mid-log anomaly is a low-prior-probability bit-flip-or-disk-issue that points at real data corruption.

**Rejected: refuse-to-start on any anomaly (paranoid mode).** Operationally annoying — every crash requires manual intervention before the server boots. Some safety-critical systems do this; for a learning-shaped distributed KV store, it's the wrong default.

**Rejected: skip-and-continue on mid-log corruption.** A PUT effect with a bad CRC would be skipped while subsequent PUT/DELETE records apply on top of a state missing that key's value. The client that received `OK` on the dropped write has no signal that its data is gone. Silent data loss is precisely the failure mode a WAL exists to prevent — much worse than a loud refuse-to-start, which surfaces the corruption to an operator who can investigate. No production WAL does this.

**Rejected entry-point alternatives.**

- **Free function `replay(path) -> impl Iterator<Item=Effect>` + manual wiring.** Caller loads effects, then constructs Storage manually. More flexible but more setup ceremony, and creates a footgun: forgetting to call `replay` produces an empty Storage that silently shadows persisted state.
- **Two-step `Storage::new()` + `storage.replay(path)?`.** Most flexible; easiest to misuse. Same footgun as above, plus an additional "did you remember" step.

The constructor form (`Storage::open`) eliminates the footgun. Idiomatic Rust constructor pattern; mirrors `File::open` + `TcpListener::bind` + countless other "construct fully-initialized resource" patterns.

### #7 — Concurrency model: inherit `Arc<Mutex<Storage>>` from M2

**Decision.** No new synchronization for the WAL. The existing outer Mutex from ADR 0013 serializes all storage operations, including WAL appends. The `File` handle inside Storage is Mutex-protected by virtue of being inside the locked struct. Per-write `sync_data()` (#4) means there is no background fsync task, and replay at `Storage::open()` runs single-threaded before the Mutex is shared with worker tasks.

**Alternatives considered (thin).**

**WAL-internal sync (e.g. inner `Mutex<File>`).** Over-engineered. The outer Mutex already guarantees single-writer.

**`tokio::sync::RwLock` from M3.** Would let parallel reads through. Minor win at M3 (read-heavy patterns aren't expected yet), and writer starvation is a real failure mode under contention. Deferred until benchmarks show it matters.

**Sharded storage from M3 (`Arc<[Mutex<Shard>]>`).** Real scale-out pattern. Defers naturally to M8 partitioning, which is the principal scaling axis. Redis Cluster does this. Doing it at M3 would conflate two milestones for no clear benefit.

What we lose: reads block writes and vice versa. Acceptable for M3 — workload is bounded. Behaviorally close to Redis, which is single-threaded for command execution; reads also block writes there, just by virtue of one event loop. Real scale-out in the kintoun arc is via partitioning at M8, not via fancier locking.

## Consequences

- **Persistence by construction.** Once M3 ships, every acked SET/INCR/DEL is on disk before the client sees `OK`. Restart preserves state.
- **`Storage` is the durability boundary.** Code outside the storage module need not know whether a given operation will hit disk; that's a configuration of the type, not an architectural variant.
- **Contract suite proves equivalence.** The 22 contract tests run in two configs (no-WAL + WAL-backed). Behavioral drift between modes is caught automatically. The M1 investment in trait + contract pattern continues to earn.
- **Per-write fsync caps single-stream throughput at fsync latency.** ~1ms on SSD, ~5–20ms on slower disks. This is the target throughput at M3; group commit becomes a knob to turn at M5+ if real workloads pressure it.
- **The on-disk format survives the milestone arc.** Per-record seq# enables M4 (subscribers cursor by seq#), M5 (replicas request "everything after seq N"), and M7 (Raft log indexing). CRC enables silent-corruption detection across all milestones. The format is forward-compatible with segmentation at M6 (single file = one segment, no record changes).
- **`Effect` becomes a public storage type.** Effects are visible at the trait surface (and ship over the network at M5). Designed accordingly: small, bytes-clean, serializable.
- **`fsync()` errors are propagated to the caller as `ERR` frames at M3.** The fsyncgate-correct behavior (panic on fsync error, replay on restart) is gated on restart-recovery infrastructure we don't have yet. Captured as Open Follow-up.
- **`tail -f wal.log` is partially debuggable.** The 16-byte binary header is opaque, but the UTF-8 effect lines are greppable (`grep PUT wal.log` mostly works, modulo header bytes preceding payloads). A small helper script `xxd wal.log | less` is enough for hands-on inspection at M3; a dedicated debug tool is a stretch.
- **Drift risk is single-source.** One `Storage` impl means one truth for HashMap + WAL semantics. The two-config contract suite catches behavior differences between modes; there is no second impl to drift against.
- **Reads still block writes.** Acceptable at M3; the principal scaling axis is partitioning at M8, not RwLock at any earlier milestone.

## Open Follow-ups

*These are forward-looking concerns deferred to later milestones or specific conditions — not unresolved M3 design questions. The M3 design phase is closed; revisits go through new ADRs or status notes.*

- **fsyncgate handling.** On Linux, a single `fsync()` error compromises subsequent writes' durability semantics — the kernel may discard the failed dirty pages and operate as if the file is clean. PostgreSQL, etcd, and MongoDB all hit this in 2018; the consensus fix is panic-on-fsync-error with replay on restart. M3 propagates fsync errors to the caller; the panic-and-replay path is gated on restart-recovery infrastructure that doesn't exist yet. Revisit when that infrastructure lands.
- **Group commit / batched fsync.** Throughput optimization for high-write-rate workloads. Add at M5+ if real workloads pressure single-stream fsync latency.
- **Configurable fsync policy.** Mirror Redis's `appendfsync` modes (`always` / `everysec` / `no`) once there's a real reason to weaken durability for throughput. M5+ at earliest.
- **File segmentation.** Switch from single growing file to size-rotated segments at M6 when compaction (drop subsumed records) and retention (drop oldest data) become first-class concerns. Per-record seq# means no record-format change is required.
- **Compaction.** Per-key last-write-wins compaction at M6+. Maps cleanly to segments — rewrite N old segments into 1 compacted segment, drop originals.
- **`apply(Effect)` at the trait surface for replication.** M5 followers receive effects from the leader and must apply them to local state. The primitive already exists internally at M3 — the replay loop applies effects to a fresh map record-by-record. M5's work is to promote that internal helper to a public trait method and route inbound network effects through it. Reframes the M5 refactor as "expose what already works" rather than "design + implement an effect-application path."
- **`RwLock` or sharded storage.** Add when benchmarks show the Mutex contended. The trait abstraction makes the swap mechanical at the call sites.
- **Debug / inspection tool.** A small `kintoun-wal-dump` binary that reads `wal.log` and prints (seq, effect) pairs. Useful for incident response and for the M4–M5 milestones when WAL contents become operationally interesting. Defer until the need surfaces.
- **Coverage threshold ratchet.** Per ADR 0011, set the line-coverage threshold once the M3 baseline stabilizes. Storage gains substantial new code paths; measure first.
