# ADR 0002: Crate Name — kintoun

Date: 2026-05-01
Status: Accepted

## Context

A publishable crate name was needed before any release work. The name appears in `Cargo.toml`, on crates.io, and in every `use` statement downstream. It is high-visibility and effectively permanent once published.

The original candidate was `nimbus` — a reference to Goku's Flying Nimbus from Dragon Ball, fitting the project's distributed-cloud theme. That name was already taken on crates.io.

## Decision

Use **`kintoun`** (筋斗雲 — the Japanese original for Goku's Flying Nimbus). The project directory was renamed from `/home/netrom/nimbus` to `/home/netrom/kintoun` on the same date to match. The GitHub repository `mauv0809/kintoun` was aligned. Future hosted version reserved as `kintoun.cloud`.

Published as `kintoun@0.0.1` on 2026-05-01 — `0.0.x` series chosen over `0.1.0` to honestly signal pre-alpha. The publish was for name reservation, not feature-completeness.

## Consequences

- The name is distinctive; collision risk on crates.io is effectively zero.
- The Dragon Ball "flying cloud" reference of the original `nimbus` candidate is preserved, just in the source language.
- Directory, GitHub repo, crate name, and reserved domain (`kintoun.cloud`) are all aligned. No naming drift across surfaces.
- Pronunciation is less familiar to English-speaking users than `nimbus` would have been. Likely variants in the wild: "kin-TOON," "KIN-toh-oon," "kin-toh-OON." This is accepted.
- Pre-2026-05-01 documentation, commit messages, and notes reference `nimbus`. They are not retroactively rewritten; readers must read those in historical context.
