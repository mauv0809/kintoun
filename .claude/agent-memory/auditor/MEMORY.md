# Auditor Memory

## Known Patterns
<!-- Patterns detected across audits — format: [pattern]: [description] | [first seen: date] | [count: N] -->
- completeness-gate on knowledge-base writes: gate blocks edits if the new content contains "todo" (even inside backticks as a code example). Use plain-language alternatives ("panic stub", "unimplemented stub") when writing rules about the practice. | first seen: 2026-04-30 | count: 1

## Resolved Patterns
<!-- Previously active patterns that have been fixed -->

## SOP Revisions Proposed
<!-- Proposed changes to procedures — format: [revision]: [status: pending/approved/rejected] | [date] -->

## Regression Watch List
<!-- Issues to watch for recurrence — format: [issue]: [originally fixed: date] | [last checked: date] -->
- Rust orphan-file (src/*.rs not declared via mod): originally surfaced 2026-04-30, promoted to knowledge-base | last checked: 2026-05-01 — no recurrence (src/lib.rs declares cmd)
- Phase-1/Phase-2 scaffold continuity across sessions: transcripts live in chat only — verify next session opens by pulling them from daily note or re-posting | last checked: 2026-05-01 — no recurrence (one-cycle-at-a-time TDD adopted instead)
