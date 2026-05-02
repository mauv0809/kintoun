# ADR 0001: Toolchain, Edition, and Licensing

Date: 2026-04-30
Status: Accepted

## Context

Greenfield Rust project. The primary outcome is learning Rust by building a distributed KV store with stream/queue/cluster ambitions through M8. Foundational toolchain and licensing choices need to be locked before code lands, because changing any of them later (edition, MSRV, licensing) ripples through the entire project.

## Decision

- **Rust edition: 2024.** Latest stable edition at project start.
- **MSRV: 1.85.** First stable release supporting the 2024 edition. Recorded in `Cargo.toml` (`rust-version = "1.85"`) and `clippy.toml` (`msrv = "1.85"`).
- **License: MIT OR Apache-2.0.** Dual-licensed per the Rust ecosystem norm. `LICENSE-MIT` and `LICENSE-APACHE` files at the crate root; `license = "MIT OR Apache-2.0"` in the manifest.
- **Dependencies:** `thiserror = "2"` (derive `std::error::Error`), `anyhow = "1"` (top-level error type in `main.rs`).
- **Dev dependencies:** `proptest = "1"` (property tests), `pretty_assertions = "1"` (better diff output on `assert_eq!` failures).
- **CI:** GitHub Actions on `ubuntu-latest`, stable Rust via `dtolnay/rust-toolchain@stable` with `rustfmt` + `clippy` components. Build cache via `Swatinem/rust-cache@v2`. Three steps: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`. Triggers: every push to `main`, every PR.
- **Format/lint config:** `rustfmt.toml` pins `edition = "2024"` + `max_width = 100`. `clippy.toml` pins `msrv = "1.85"`.

## Consequences

- The crate floor is Rust 1.85 — anyone on an older toolchain cannot build it. Acceptable given M1's start date and the project's modern ambitions.
- Dual licensing matches the Rust ecosystem default. Downstream users (and the future hosted version `kintoun.cloud`) can pick whichever fits their context.
- The `thiserror` / `anyhow` split (per-module derived errors, top-level dynamic) is the conventional Rust pattern. Choosing both upfront avoids per-module debate as new modules land.
- CI catches formatting drift, lint warnings, and broken tests before merge. Coverage is intentionally narrow at M1 — no security audit, no doc build, no MSRV cross-check, single OS, single toolchain. These layer in as the project earns them.
- The MSRV declarations in `Cargo.toml` and `clippy.toml` must stay synchronized. Bumping one without the other creates silent drift: clippy may suggest features the declared MSRV doesn't support, or fail to flag features that exceed it. Both fields move together in the same commit.
