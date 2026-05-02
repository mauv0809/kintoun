# ADR 0004: Error Model — Per-module Errors with Educational Hand-Roll

Date: 2026-04-30 (initial design); 2026-05-01 (ParseError landed; hand-roll → thiserror conversion)
Status: Accepted

## Context

Rust has multiple conventions for error handling:

- **Single global error enum** — every module's errors funnel through one type. Simple at the top level; bloated and misleading per-module (a function that can only fail for parser reasons appears to be able to fail for storage reasons too).
- **Per-module errors** — each module defines its own enum. Type signatures are honest; cross-module composition needs `From` impls.
- **Fully dynamic** (`Box<dyn Error>` / `anyhow::Error` everywhere) — composition is trivial; type-level information is discarded.

The project also faces a secondary consideration: `thiserror` (a derive macro that generates `impl Display` + `impl std::error::Error`) makes error types essentially free to write. Using a derive macro from day one means never seeing what the macro does. For a Rust learning project, that is a missed pedagogical opportunity.

## Decision

**Per-module error types.** Each module owns its own error enum.

- `cmd::ParseError` — covers parser failure modes: `Empty`, `MissingArg`, `UnknownCommand`, `InvalidAmount`, `TooManyArgs`.
- `storage::StorageError` — will cover storage failures (TBD as `storage.rs` lands).
- Top-level `main.rs` returns `Result<(), anyhow::Error>` for ergonomic propagation across module boundaries.

`anyhow` discards type information at the binary edge; per-module types preserve it inside the library. This is a deliberate split: type-information matters for callers who want to handle specific failures; it does not matter at `main.rs`, where the only consumers are exit codes and printed messages.

**Educational hand-roll for `ParseError`, then conversion to `thiserror`.**

First iteration (2026-05-01): hand-write `impl Display` + `impl std::error::Error for ParseError {}` to learn the trait shape, the `Formatter` machinery, and the supertrait bounds. Second iteration: convert to `#[derive(thiserror::Error)]` with `#[error("...")]` attributes per variant. The hand-rolled version does not survive in the final code — its purpose was to make the macro's expansion legible.

**Asymmetric variant payloads.** Within `ParseError`:
- `&'static str` for hardcoded labels under parser control (`MissingArg("key")`, `TooManyArgs("get")`). Zero allocation; the data is a literal the parser owns.
- `String` for user-input echoed back (`UnknownCommand(verb.to_string())`). Allocation needed; the data comes from the caller.
- Struct variant when multiple pieces of context matter (`InvalidAmount { input, reason }`).

This pattern is deliberate, not inconsistent.

**Variant-shape rubric for new variants:**
- Multiple fields → struct variant always.
- Single field where the variant name implies the role → tuple variant (terser, less ceremony).
- Single field where the field name adds critical context → struct variant.

## Consequences

- Module API surfaces are honest about what *that module* can fail at. Readers don't have to scan a 50-variant global error to understand a single function's failure modes.
- `?` lifts cleanly: each module's error → `anyhow::Error` at `main.rs` via `From` impls (auto-provided by `thiserror`'s derive when the source error implements `std::error::Error`).
- The hand-roll → `thiserror` transition is recorded in commit history and this ADR. Future readers can trace the pedagogical path. Without ADR/commit context, the surviving code looks like "always used `thiserror`," which would obscure the learning intent.
- When a new module appears, the same pattern repeats. Boilerplate cost per module is 5-10 lines of derive + variants.
- `anyhow` at the top level loses type-level error information. Acceptable because `main.rs` is the binary edge.
- The asymmetric-payload pattern means readers must understand *why* one variant uses `&'static str` and another uses `String`. The rubric in this ADR is the explanation; comments in the code are not used.
