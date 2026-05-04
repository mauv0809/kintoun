# Development Setup

Local checks that match what CI enforces. Set up once per clone.

## Toolchain

```sh
rustup default stable
rustup component add rustfmt clippy llvm-tools-preview
```

## Required local tools

```sh
cargo install bacon            # background watcher (file-saves trigger checks)
cargo install cargo-deny       # license + advisory policy (mirrors CI)
cargo install cargo-llvm-cov   # coverage measurement (mirrors CI)
```

## Three layers of formatting checks

CI runs `cargo fmt --all -- --check` on every push. To catch format issues before they get there:

### 1. IDE format-on-save (catches at edit time)

**VS Code** — install the `rust-analyzer` extension and add to `.vscode/settings.json` (or your user `settings.json`):

```json
{
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

**JetBrains** (RustRover / IntelliJ + Rust plugin): Settings → Rust → Rustfmt → check "Run rustfmt on Save."

**Neovim** (with rust-analyzer via LSP):

```lua
vim.api.nvim_create_autocmd("BufWritePre", {
  pattern = "*.rs",
  callback = function() vim.lsp.buf.format() end,
})
```

### 2. bacon (catches at file-save time)

`bacon.toml` defines a `fmt` job. From the project root:

```sh
bacon fmt
```

bacon will re-run `cargo fmt --check` on every save and show the diff if formatting drifts.

For continuous correctness checking during normal coding, `bacon` (no args) runs `cargo check` on save. Switch jobs with `bacon clippy`, `bacon test`, `bacon fmt`.

### 3. Git pre-commit hook (catches at commit time)

A pre-commit hook lives in `.githooks/pre-commit`. Enable it for your clone:

```sh
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit
```

After enabling, every `git commit` runs `cargo fmt --check` first and aborts if the diff isn't clean.

## Other CI-parity checks

```sh
cargo clippy --all-targets --all-features -- -D warnings   # CI: Clippy
cargo deny check                                            # CI: cargo deny
cargo llvm-cov --all-features --workspace --summary-only    # CI: Test + Coverage
```

Run any of these locally to preflight before pushing.
