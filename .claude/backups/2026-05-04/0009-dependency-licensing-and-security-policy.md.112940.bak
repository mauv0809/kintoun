# ADR 0009: Dependency Licensing and Security Policy

Date: 2026-05-04
Status: Accepted

## Context

`kintoun` is a published crate (crates.io) dual-licensed `MIT OR Apache-2.0` (ADR 0001). It depends transitively on crates we don't author. Two recurring questions need a policy answer that doesn't get re-litigated on every PR:

1. **License compatibility.** Which transitive licenses are compatible with our dual MIT/Apache-2.0 stance? A copyleft transitive dep (GPL/AGPL) would force `kintoun` itself to be copyleft, breaking our license promise.
2. **Security advisories.** RustSec maintains an advisory database for Rust crates. Without an automated check, a vulnerable transitive dep can sit in `Cargo.lock` until someone notices manually.

A future commercial fork — `kintoun.cloud` — also wants to inherit a clean license posture. Decisions made now should not block that.

### Alternatives Considered

**A — `cargo audit` only.** Single-purpose RustSec advisory checker. Smaller surface, but doesn't cover license policy. Would require a second tool for licenses, doubling config and CI cost.

**B — Manual review at each `cargo update`.** Read every transitive license, scan RustSec by hand. Doesn't scale; humans skip steps under deadline pressure.

**C — `cargo deny` (chosen).** Covers advisories + licenses + banned crates + source restrictions in one config + one CI step. Superset of `cargo audit`'s functionality. Single source of truth in `deny.toml`.

## Decision

1. **Tool: `cargo-deny`**, configured via `deny.toml` at the repo root. Run via `cargo deny check` in CI on every push and PR.

2. **Allowed licenses — standard permissive:**
   - `MIT`, `Apache-2.0` (+ `WITH LLVM-exception`), `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Unicode-3.0`, `Unicode-DFS-2016`, `Zlib`, `0BSD`, `BSL-1.0`.
   - Implicitly denied: `MPL-*`, `LGPL-*`, `GPL-*`, `AGPL-*`, and anything else not on the allow list.

3. **Advisories.** v2 defaults — vulnerabilities are denied automatically. `yanked = "warn"` flags yanked crates without blocking. `ignore = []` — every advisory is reviewed when it appears, never silently suppressed.

4. **Bans.** `wildcards = "deny"` — no `version = "*"` in `Cargo.toml`. `multiple-versions = "warn"` — duplicate transitive versions are noise, not failure. `deny = []` — populated per incident if a specific crate becomes problematic.

5. **Sources.** `unknown-registry = "deny"`, `unknown-git = "deny"`. crates.io only.

## Consequences

- Every PR runs `cargo deny check`. License or advisory regressions fail CI before merge.
- Adding a new dep with an off-list license produces an actionable error ("license X for crate Y is not allowed"). The fix is either to find an alternative dep or to add the license to the allow list — the latter requires a deliberate `deny.toml` edit, making policy shifts visible in diff review.
- New RustSec advisories surface within one CI run of publication.
- The permissive-only stance keeps `kintoun.cloud` unblocked: any future commercial fork inherits a codebase free of copyleft transitive obligations.
- `deny.toml` becomes a small, durable surface that shifts policy decisions out of code review (where they get missed) and into a single reviewable file.
- One CI step adds roughly 20–40 seconds — negligible against the existing test job.
- `cargo deny check` runs locally too. Contributors can preflight before pushing.

## Open Follow-ups

- **License clarifications.** Some crates have ambiguous license text and need a `[[licenses.clarify]]` entry. Add reactively as `cargo deny check` flags them — not preemptively.
- **Vendored binaries.** `[bans.build]` rules (executables, interpreted scripts) are not configured at M1. Reconsider when the dep graph grows or we add deps known to ship binaries.
- **Multiple-versions hardening.** Currently `warn`. Consider promoting to `deny` once the dep graph stabilises, with a `skip-tree` allowlist for foundational crates that legitimately ship multiple versions (e.g. `windows-sys`).
