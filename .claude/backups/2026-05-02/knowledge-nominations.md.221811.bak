# Knowledge Nominations

Candidate learnings from agents and sessions. The auditor reviews these
during each audit cycle and promotes valid ones to knowledge-base.md.

## Pending Nominations

- [043026] /wrap-up: **Use the type system to validate input where possible.** `s.parse::<u64>()` rejects negatives for free. Choosing `u64` over `i64` for `Incr.by` shifted the "no negative amounts" rule from a runtime check into a parse-time error message — fewer code paths, clearer errors, no manual branch needed. | Evidence: agent inference 043026 — applied during `Incr`/`Decr` field-type decision; user confirmed the semantic preference ("incr by negative is weird"). | Status: DEFERRED — one confirmed application; promote after a second instance surfaces.

- [050126] /wrap-up: **`?` propagates from the current function/closure only — it cannot escape closure boundaries.** Putting `?` inside an `and_then(|s| ...)` closure tries to early-return from the closure, not the enclosing function. The closure's return type (`Option<U>`) and the outer function's return type (`Result<_, _>`) are independent. Workaround: lift the fallible step out of the closure into the function body (use `match parts.next() { Some(s) => s.parse()?, None => default, }`). | Evidence: empirical 050126 — compile error E0277 on `parts.next().and_then(|s| s.parse::<u64>()?).unwrap_or(1)` during `incr` arm work. | Status: PROMOTED 050126.

- [050126] /wrap-up: **Local type inference can fail at closure boundaries when generic methods meet integer-literal ambiguity.** `s.parse().map_err(|e| e.to_string())` won't compile when the eventual target is `u64` — because `1` in a sibling `match` arm keeps `T` as `{integer}` until later, and `T::Err` can't be resolved at the closure body. Fix: pin the target type at the `parse()` site with turbofish (`s.parse::<u64>()`). General rule: when inference stalls with "type annotations needed" inside a closure, annotate the generic call upstream, not the closure parameter. | Evidence: empirical 050126 — compile error E0282 on `s.parse().map_err(|e| ParseError::InvalidAmount { reason: e.to_string(), ... })`. | Status: PROMOTED 050126.

- [050126] /wrap-up: **`From<E1> for E2` cannot capture calling-scope context — it only sees the source error.** When you need an error variant to carry context that's only available at the call site (e.g. the bad input the user typed), drop the `From` impl and use `.map_err(|e| Variant { input: s.to_string(), reason: e.to_string() })?` at the site. Trade auto-`?`-via-`From` for richer error data. | Evidence: agent inference 050126 — applied during `InvalidAmount { input, reason }` design after recognizing `From<ParseIntError>` could only access the `ParseIntError`, not the original `&str`. | Status: PROMOTED 050126.

- [050126] /wrap-up: **Asymmetric error variants for asymmetric data sources is idiomatic Rust.** `&'static str` for hardcoded labels you control at the call site (zero alloc, no lifetime parameter), `String` for user input/runtime data, struct variant `{ input: String, reason: String }` when both pieces add value. The mix in one enum is normal and signals the variant's data shape. | Evidence: agent inference 050126 — applied across `Empty` (unit) / `MissingArg(&'static str)` / `UnknownCommand(String)` / `InvalidAmount { input, reason }` in `cmd::ParseError`. | Status: PROMOTED 050126.

- [050126] /wrap-up: **Patterns are a separate language from expressions** — no function calls, no method calls, no construction (`String::from(...)` etc.). To assert against a complex value inside a match arm, either use a guard (`pattern if expr`) or bind first and assert second. The "bind then assert" form gives the best test failure messages because `assert_eq!` produces a side-by-side diff; the guard form just produces a generic "didn't match" panic. | Evidence: empirical 050126 — `Err(ParseError::UnknownCommand(String::from("foo")))` pattern rejected; replaced with `Err(ParseError::UnknownCommand(s)) => assert_eq!(s, "foo")`. | Status: PROMOTED 050126.

## Promoted Nominations

- [043026] Rust orphan-file silent footgun → knowledge-base.md "Known Failure Modes" (empirical 043026). Promoted 043026.
- [043026] Default values in the parser, not the type → knowledge-base.md "Project Patterns" (agent inference 043026). Promoted 043026.
- [043026] Per-arm TDD discipline for match dispatchers → knowledge-base.md "Project Patterns" (agent inference 043026). Promoted 043026.
