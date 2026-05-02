# ADR 0007: M1 Command Grammar

Date: 2026-04-30 (initial design); 2026-05-02 (edge-case contract locked)
Status: Accepted

## Context

M1 ships an interactive REPL that accepts text commands, parses them into a `Command` enum, and dispatches to storage. The grammar must be small (M1's purpose is the storage and REPL machinery, not parser engineering), but well-defined — undefined behavior at the grammar level becomes silent bugs at the application level.

Three categories of edge cases were weighed in the parser design pass on 2026-05-02 — verb casing, trailing arguments, and multi-word values via quoting. Each had two reasonable answers; the chosen contract is recorded below.

## Decision

**Six command verbs:**

- `set <key> <value>` — store a value
- `get <key>` — read a value
- `del <key>` — delete a key
- `exists <key>` — test for presence
- `incr <key> [<amount>]` — increment (default amount: 1)
- `decr <key> [<amount>]` — decrement (default amount: 1)

**Tokenization:** `str::split_whitespace()`. Tabs, newlines, and runs of whitespace all collapse to a single delimiter. Leading and trailing whitespace are ignored. The choice means M1 cannot tokenize quoted strings — see the multi-word values decision below.

**Verb case sensitivity: case-insensitive.** Verbs are normalized via `verb.to_ascii_lowercase()` before matching. `SET`, `Set`, `sEt`, and `set` all parse as `Command::Set`. Keys and values remain case-sensitive — only the first whitespace-delimited token (the verb) is lowercased.

The chosen alternative was strict case-sensitivity (mirrors the in-code `match` arms; zero allocation). It was rejected after revision: case-insensitive is the convention users expect from KV CLIs (Redis, memcached), and a single allocation per parse is not a meaningful cost.

**Trailing arguments: rejected.** Any token remaining after a command's expected arguments produces `ParseError::TooManyArgs(verb)`. Example: `get foo bar` → `TooManyArgs("get")`. Enforced by an `expect_done` helper called at the end of each parser arm, before the final `Ok(Command::...)`.

The chosen alternative was silent ignore (lenient parsing). It was rejected: silent ignore would mean typos like `set foo bar baz` would silently store `bar`, and the user would see no signal that `baz` was dropped. Loud failure at the grammar level prevents that class of bug from reaching application logic.

**Multi-word values via quoting: deferred.** `set foo "hello world"` does not work in M1 — `split_whitespace()` would split the quoted string. A real tokenizer with quote/escape handling is a project of comparable scope to the rest of M1.

The chosen alternative was implementing quoting in M1. It was rejected: M1's scope is storage and REPL machinery, not parser engineering. M2's network protocol will likely use length-prefixed framing (no quoting needed), so the quoting question may dissolve before M1 finishes.

**Default values: filled in the parser, not the type.** `incr foo` (no amount) parses to `Command::Incr { key: "foo", by: 1 }`. The enum variant always has `by` populated — no `Option<u64>`. Downstream code never branches on "did the user supply this?"

**Error preservation: original-case verb in user-facing messages.** `ParseError::UnknownCommand` echoes the original-case verb (`unknown command: FOOSBALL`, not `unknown command: foosball`). Lowercasing affects matching, not user-facing output. Implemented by binding `verb` *before* the `to_ascii_lowercase()` call and using the original binding in the catch-all arm.

## Consequences

- The parser layer is feature-complete for M1. As of 2026-05-02, 28 tests cover happy paths, all five `ParseError` variants (`Empty` / `MissingArg` / `UnknownCommand` / `InvalidAmount` / `TooManyArgs`), whitespace handling, case-insensitivity, and trailing-args rejection.
- The case-insensitive policy is implemented at the parser via `to_ascii_lowercase()` (one allocation per parse, bounded by verb length — at most ~6 bytes). When M2's framing layer enters, normalization may move upstream so the parser receives canonical input. That preserves `match` exhaustiveness without runtime case folding.
- Trailing-args rejection means typos like `set foo bar baz` fail loudly. This is a deliberate trade against lenient parsers like the Redis CLI's looser handling.
- Deferring quoted strings means M1 cannot store values containing whitespace. Acceptable for M1's scope.
- The default-in-parser pattern keeps `Command` variant shape uniform — no `Option<u64>` for `by`. Pays off when downstream code (executor, storage) consumes the variants.
- Original-case error preservation requires the parser to bind `verb` before lowercasing. A naive refactor that lowercases in place would silently regress this property. The test `parse_unknown_verb_returns_unknown_command_error_keeping_case` guards against this by asserting an uppercase unknown verb (`FOOSBALL`) is echoed unchanged in the resulting `UnknownCommand` payload.
