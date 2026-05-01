# Knowledge Nominations

Candidate learnings from agents and sessions. The auditor reviews these
during each audit cycle and promotes valid ones to knowledge-base.md.

## Pending Nominations

- [043026] /wrap-up: **Use the type system to validate input where possible.** `s.parse::<u64>()` rejects negatives for free. Choosing `u64` over `i64` for `Incr.by` shifted the "no negative amounts" rule from a runtime check into a parse-time error message — fewer code paths, clearer errors, no manual branch needed. | Evidence: agent inference 043026 — applied during `Incr`/`Decr` field-type decision; user confirmed the semantic preference ("incr by negative is weird"). | Status: DEFERRED — one confirmed application; promote after a second instance surfaces.

## Promoted Nominations

- [043026] Rust orphan-file silent footgun → knowledge-base.md "Known Failure Modes" (empirical 043026). Promoted 043026.
- [043026] Default values in the parser, not the type → knowledge-base.md "Project Patterns" (agent inference 043026). Promoted 043026.
- [043026] Per-arm TDD discipline for match dispatchers → knowledge-base.md "Project Patterns" (agent inference 043026). Promoted 043026.
