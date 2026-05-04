# ADR 0008: Tagged StoredValue and Executor-Level Inference

Date: 2026-05-04
Status: Accepted (implemented in M1 — `src/storage.rs`; executor wiring pending in `src/executor.rs`)

## Context

ADR 0005 locked the apply-mutation pattern but left the value type underspecified. The original sketch had `Mutation::Set { value: String }` and `read -> Option<&str>` — sufficient for an in-memory KV demo but quietly load-bearing for the milestone arc that follows M1.

Two pressures collided during implementation:

1. **UX.** Users typing `set counter 5; incr counter` from the REPL expect the second command to succeed. With `Mutation::Set { value: String }`, every counter op would have to parse the string on each invocation — and "did this slot start life as a counter or as a string that looks like a number?" becomes indistinguishable to storage.

2. **Type honesty.** If the storage layer pretends "everything is a string," future variants (List, Hash, Stream — natural at M4–M5) end up smuggled in via stringification. Match exhaustiveness — a load-bearing design tool in this project — gets diluted because consumers cannot enumerate variants the type system doesn't expose.

The question was *where* to put the rule "this textual token is numeric." That question has architectural weight: the rule's location determines what other input sources (M2 wire frames, M3 WAL replay, future client SDKs) have to import or duplicate.

### Alternatives Considered

**A — Drop StoredValue, keep `String`-only storage; counter ops parse on demand.** Smallest surface area; maximum Redis-likeness. Rejected: loses the "this slot is a counter" type information; pays reparse cost on every counter op; reverts honest design work; most importantly, blocks M4+ variant growth (List, Hash, Stream) without a redesign.

**B — Tagged StoredValue, but storage layer auto-coerces at apply time.** `apply` would call `parse::<u64>()` whenever a `Set` arrived. Rejected: hides a heuristic behind the trait method; couples storage to a textual-input policy that storage shouldn't own; surprises direct callers (tests, internal code) who built a `Str("5")` and didn't expect silent coercion.

**C — Tagged StoredValue + inference at executor boundary (chosen).** Storage layer strict; executor layer opinionated. Inference rule named, lives next to the type, called from the boundary layer.

**D — Tagged StoredValue + inference in the parser.** `cmd.rs` would import `StoredValue` and produce `Command::Set { value: StoredValue }` directly. Rejected: forces every input source (M2 wire frames, future RPC) to either share the rule via storage import or duplicate it; mixes parser concerns (text → tokens) with semantic concerns (token → typed value). A short-lived branch attempted this and was reverted on 2026-05-04.

## Decision

1. **Storage carries tagged values.**

   ```rust
   pub enum StoredValue {
       Str(String),
       Int(u64),
   }
   ```

   `Mutation::Set { key: String, value: StoredValue }`. `Storage::read -> Option<StoredValue>` (owned).

2. **Storage is strict — no coercion at apply time.** `Set` stores whatever variant it was given. `Incr` and `Decr` on `Str(_)` return `StorageError::NotAnInteger` and preserve the original value. The test `apply_incr_on_numeric_string_errors` asserts this invariant directly: a `Str("5")` followed by `Incr` errors, even though the string parses as a `u64`.

3. **Inference is named and lives next to the type.** `StoredValue::from_text(s: &str) -> Self` is the single point where text becomes typed. Numeric tokens that `parse::<u64>()` accepts become `Int`; everything else becomes `Str`. Negatives, decimals, hex prefixes, plus prefixes, overflow, and empty strings all fall through to `Str`.

4. **The executor calls `from_text` at the boundary.** Parsed `Command`s carry `String` values; the executor invokes `StoredValue::from_text(&value)` once per `Set` to produce the `Mutation`. M2's wire-protocol framer will do the same. `cmd.rs` does not import `StoredValue`.

## Consequences

- `set foo 5; incr foo → 6` works through the REPL path because the executor coerces `"5"` → `Int(5)` once on entry.
- Direct callers that build `StoredValue::Str("5")` see strict semantics: a subsequent `Incr` errors. Two coherent invariants for two coherent layers.
- `MutationOutcome` shape is now concrete: `Stored | Deleted | Counter { new_value: u64 }`. ADR 0005 deferred this; it is locked here.
- The 8 tests around `from_text` are the contract for the inference rule. Policy shifts (e.g., add float support, accept hex) require touching those tests, making the change intentional rather than incidental.
- M3 WAL serialization becomes a tagged union — one byte tag per `StoredValue` variant on the wire. Symmetric with in-memory layout; replay is a straight deserialization, no string-to-int reparse.
- M4 streaming subscribers see typed events. No "is this number a number" guesswork at the consumer.
- Future variants (`StoredValue::List(Vec<…>)`, `Hash(HashMap<…>)`, `Stream(…)`) slot in additively. Match exhaustiveness forces every consumer to handle them, which is the desired forcing function.
- This ADR supersedes the `read -> Option<&str>` and `Mutation::Set { value: String }` signatures from ADR 0005's Decision section. The apply-mutation split itself is unchanged.

## Open Follow-ups

- **Quoted tokens (deferred post-M1).** When quoting lands, decide whether quoted tokens skip inference (`set foo "5"` → always `Str("5")`) or still go through it (quoting handles whitespace only). Today's decision doesn't bind that one; expect a follow-up ADR.
- **Signed integers / floats.** If the milestone arc grows numeric ambitions beyond unsigned counters, `from_text` and `StoredValue` extend together. Not a forcing function today.
- **WAL serialization format.** M3 will pick a wire format for the tagged union (likely a one-byte tag + length-prefixed payload). Out of scope here; ADR at M3.
