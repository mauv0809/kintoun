# ADR 0010: REPL Output Format and Error Handling Conventions

Date: 2026-05-04
Status: Accepted

## Context

M1 closes with `src/repl.rs` exposing kintoun to a human via stdin/stdout. The format choices made here outlive M1:

- **M2 wire protocol** may reuse or diverge from these conventions; either way it needs to know what was decided here and why.
- **Future quoted-input parsing** (deferred post-M1) raises escape-rule questions that depend on today's Str rendering choice.
- **Error display conventions** become muscle memory once shipped; picking weird ones now means re-training later.
- **Boundary vs layer error handling** — repl.rs is the first place where errors from multiple library layers (`ParseError`, `ExecuteError`) need to converge. The choice here sets a precedent for M2+ where async/tokio brings more error types.

Two decision clusters: output format for `ExecuteResult` rendering, and error unification at the REPL boundary.

### Alternatives Considered — output format

**A — Redis-like (chosen).** Battle-tested KV-CLI conventions: `OK` for mutations, raw counter values, `(nil)` for missing reads, quoted Str values, `1`/`0` for existence. Compact, distinguishes edge cases (`(nil)` vs `""`).

**B — Verbose/explicit.** `OK`, raw counters, `nil`, unquoted strings, `true`/`false`. Easier first-time UX; introduces ambiguity between `foo` the verb token and `foo` the unquoted string value.

**C — Minimal/raw.** Empty line for success-without-value, value-only output for reads, `true`/`false` for existence. Most pipe-friendly. Loses the distinction between "key missing" and "key set to empty string" — both render as blank.

### Alternatives Considered — error unification at REPL boundary

**A — Manual `ReplLineError` enum with `#[from] ParseError` + `#[from] ExecuteError`.** Mirrors the project's existing hand-rolled error pattern (`ExecuteError` did this for `StorageError`). ~6 lines of new type code.

**B — `anyhow::Error` via private `process_line` helper (chosen).** Zero new types. anyhow already in `Cargo.toml`. Appropriate at the REPL boundary because errors here are *displayed*, not pattern-matched-on by callers.

**C — Keep the nested two-level match in `run`.** No abstraction. Honest about the layering. Two arms of similar error display code; small win for flatness, small cost for not abstracting.

## Decision

### 1. Output format (Redis-like)

| `ExecuteResult` variant | Rendered as |
|---|---|
| `Mutation(MutationOutcome::Stored)` | `OK` |
| `Mutation(MutationOutcome::Deleted)` | `OK` |
| `Mutation(MutationOutcome::Counter { new_value })` | bare number, e.g. `5` |
| `Read(None)` | `(nil)` |
| `Read(Some(StoredValue::Str(s)))` | `"<s>"` — surrounding ASCII double quotes, no escaping at M1 |
| `Read(Some(StoredValue::Int(n)))` | bare number |
| `Existence(true)` | `1` |
| `Existence(false)` | `0` |

Each command's output is one line, followed by `\n`. The prompt `> ` precedes every read attempt, including the one immediately before EOF. All output goes to stdout.

### 2. Error display

Both `ParseError` and `ExecuteError` render to stdout with the `ERR ` prefix, followed by the underlying error's `Display` impl (provided by thiserror's `#[error("...")]` attributes). Stderr is not used at M1 — single-stream output keeps tests trivial. Revisit if production usage demands stream separation.

### 3. Error unification — `anyhow` at the boundary

The REPL uses `anyhow::Error` internally via a private `process_line` helper. Library layers (`cmd`, `storage`, `executor`) keep their hand-rolled thiserror enums.

This codifies a project-wide convention:

- **`thiserror` at library layers.** Callers may want to match on specific variants (`StorageError::NotAnInteger` for retry logic; `ParseError::TooManyArgs` for syntax-aware suggestions). Concrete enum types preserve that.
- **`anyhow` at boundaries.** Code where errors are displayed, logged, or otherwise terminal — not propagated for inspection. Boundary code is where multiple layered error types converge; anyhow's blanket `From<E: Error>` impl flattens them into one `?`-friendly type without manual wrapper enums.

### 4. Formatter location

`format_result(&ExecuteResult) -> String` is a private function in `repl.rs`. Not a method on `ExecuteResult`. The format is REPL-specific; M2's wire-protocol framer will format differently (binary frames, length-prefixed payloads, etc.). Binding the format to `ExecuteResult` would constrain it.

### 5. Quit semantics

EOF only at M1. No `quit` / `exit` verb. Ctrl+D ends an interactive session; tests rely on the input slice being exhausted. If friction surfaces, adding a verb is mechanical (one `cmd.rs` arm + a return signal from `run`'s loop).

## Consequences

- **M2 wire framer is unconstrained.** It can adopt the Redis-like format if there's reuse value, or design its own (e.g., RESP-style binary). `ExecuteResult` is shared; formatting is not.
- **Future quoted-input parsing requires Str escaping.** A stored value containing `"` would render today as `"\"foo\""` — visually broken. M1 grammar can't produce such values via input (whitespace-split tokens), but post-M1 quoting opens that door. Pick an escape policy (Rust-style `\"`, JSON-style, or RESP rules) when quoting lands.
- **Error stream consistency.** All output (results + errors + prompts) goes to stdout. Tests capture one stream. Pipe-friendly for `kintoun < commands.txt > output.txt`. The cost: stderr would be the conventional choice for diagnostic separation; we'll revisit if real usage shows it matters.
- **The thiserror-vs-anyhow split scales to M2.** Async / tokio code at M2 brings new error types (IO errors from sockets, framing errors, timeout errors). This convention says: each layer gets its own thiserror enum; any boundary unifying them uses anyhow.
- **The 11 repl tests in `src/repl.rs` lock the format.** Format tweaks require touching those tests, making changes intentional rather than incidental.
- **No quit verb is a non-decision.** It can be added in 5 minutes if needed; not adding it costs nothing today.

## Open Follow-ups

- **Str escaping rules** — pick when post-M1 quoting lands. Likely Rust-style escapes (`\"`, `\\`, `\n`) for symmetry with stored values that may contain newlines.
- **Stdout vs stderr** — revisit if production telemetry demands separation. M1 single-stream is a pragmatic default, not a principled stance.
- **Quit verb** — add if interactive friction surfaces (e.g., users habitually typing `quit` and getting `ERR unknown command`). One-line cmd.rs change + a `Done` outcome variant in repl.
- **Format-parity with M2 wire protocol** — when M2 lands, decide whether to share the human-format conventions or design fresh. This ADR doesn't constrain that decision; it documents the M1 choice for context.
