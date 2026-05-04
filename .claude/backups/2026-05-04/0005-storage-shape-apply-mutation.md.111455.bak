# ADR 0005: Storage Shape — Apply-Mutation Pattern

Date: 2026-04-30
Status: Accepted (design — implementation pending in M1)

## Context

A storage layer for a KV store can take several shapes. The choice constrains how M3 (WAL persistence + replay), M4 (pub/sub on the log), M5 (consumer groups + offsets), and M6 (single-leader replication) integrate later. Picking the wrong shape early forces invasive rework when the log enters in M3.

The key forward-looking constraint: from M4 onward, the durability log is no longer just for crash recovery. It is the substrate that pub/sub iterates over and consumer groups offset into. Whatever shape the M1 storage layer takes must produce events that are loggable, streamable, and replayable.

Several shapes were weighed during the M1 design pass; the alternatives were rejected directionally rather than via measurement, and their specifics are not preserved in working notes. The selected shape is referred to internally as "Shape B" — the apply-mutation pattern.

### Alternatives Considered

The exact shapes weighed in the M1 design pass are not preserved verbatim in working notes. The selected shape (Shape B) is documented in the Decision section below; the characterization of "Shape A" here is reconstructed from informal recollection plus standard storage-layer design patterns.

**Shape A — uniform command path.** All operations (reads and writes alike) go through a single trait method that takes a `Command` value:

```rust
trait Storage {
    fn execute(&mut self, cmd: Command) -> Result<Outcome, StorageError>;
}
```

The trait surface is uniform — one method, one input type, one output type. Internally, the implementation branches: reads do hashmap lookups, writes mutate state.

**Why Shape B was chosen over Shape A:** the asymmetry between reads and mutations is structural, not incidental, in the milestone arc. Mutations must be loggable (M3), streamable (M4), replicable (M6), and consensus-governed (M7). Reads do not. Shape A's uniform interface forces every later layer to filter reads out of the operation stream — repeating the same logic at M3, M4, M5, M6, M7. Shape B encodes the read/mutation split in the type system once, at the storage trait level. Every later layer operates on `Mutation` directly, with no filtering needed.

Shape A is also less honest about what it offers — its uniform interface suggests every operation is loggable/replicable, which is not true.

## Decision

Storage operations split into two paths: **mutations go through `apply`**; **reads bypass it**.

```rust
enum Mutation {
    Set { key: String, value: String },
    Del { key: String },
    Incr { key: String, by: u64 },
    Decr { key: String, by: u64 },
}

enum MutationOutcome {
    // exact shape deferred to implementation; one variant per Mutation
}

trait Storage {
    fn apply(&mut self, mutation: Mutation) -> Result<MutationOutcome, StorageError>;
    fn read(&self, key: &str) -> Option<&str>;
    fn exists(&self, key: &str) -> bool;
}
```

Reads (`read`, `exists`) do **not** go through `Mutation`. They are not loggable, replicable, or replayable — they don't change state.

M1 implementation: `InMemoryStorage` over `HashMap<String, String>`.

## Consequences

- M3 WAL becomes structurally trivial: every `Mutation` accepted by `apply` gets serialized and appended to the log before the in-memory state updates. Replay on startup is "iterate the log, call `apply` on each entry."
- M4 pub/sub iterates the same `Mutation` log. Streaming subscribers see the same events that durability sees — no shape mismatch.
- M5 consumer groups track byte/sequence offsets into the same log.
- M6 replication ships the `Mutation` stream from leader to followers. The follower's `apply` is the same function as the leader's.
- M7 (Raft-lite) reaches consensus on which `Mutation`s get appended; the storage layer is unchanged.
- M8 (partitioning) gives each partition its own log, but the `apply` interface stays per-partition.
- Reads stay fast — bypassing `Mutation` means no log overhead on the read path. A KV store's read:write ratio is typically read-heavy, so this matters.
- The cost is a non-trivial `Storage` interface: `apply` returns a `MutationOutcome` enum rather than `()`. Callers must handle the outcome variants. Necessary for `incr`/`decr` (which return the new numeric value) and `del` (which may indicate whether the key existed).
- `MutationOutcome`'s exact shape is deferred to implementation. The design only locks that mutations and reads are split, and that mutations go through one funnel.
- This ADR will need a status update (and possibly a new ADR) when `storage.rs` lands and the `MutationOutcome` shape is concrete.
