//! kintoun binary entry point.
//!
//! Two modes, dispatched by CLI:
//!   - no args            → REPL (sync, stdin/stdout, since M1)
//!   - `--bind <addr>`    → TCP server (async, since M2)
//!
//! Argument parsing is hand-rolled (no `clap` dep — see ADR 0013). The
//! tokio runtime is built lazily, only on the server branch, so the REPL
//! stays runtime-free.

use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, bail};
use tokio::net::TcpListener;

use kintoun::repl;
use kintoun::server::serve;
use kintoun::storage::InMemoryStorage;

/// What the binary should do this run.
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    /// Sync REPL on stdin/stdout.
    Repl,
    /// Async TCP server bound to the given address.
    Server(SocketAddr),
}

/// Parse argv into a `Mode`.
///
/// Grammar:
///   <args> ::= <binary>                         → Mode::Repl
///            | <binary> "--bind" <addr:port>    → Mode::Server(addr)
///
/// Anything else is an error. There is no `--bind` shorthand without an
/// address — bare `--bind` errors with "missing argument".
fn parse_args(args: Vec<String>) -> Result<Mode> {
    let mut iter = args.into_iter().skip(1);
    match iter.next() {
        None => Ok(Mode::Repl),
        Some(arg) if arg == "--bind" => {
            let addr_str = iter
                .next()
                .context("--bind requires an address (e.g 127.0.0.1:4242)")?;
            let addr = addr_str
                .parse()
                .with_context(|| format!("invalid socket address: {addr_str}"))?;
            if iter.next().is_some() {
                bail!("unexpected extra arguments after --bind <addr>");
            }
            Ok(Mode::Server(addr))
        }
        Some(other) => bail!("unknown argument: {other}"),
    }
}

fn main() -> Result<()> {
    match parse_args(std::env::args().collect())? {
        Mode::Repl => run_repl(),
        Mode::Server(addr) => run_server(addr),
    }
}

/// Sync REPL — same as M1's main shim.
fn run_repl() -> Result<()> {
    let stdin = io::stdin().lock();
    let stdout = io::stdout().lock();
    let mut storage = InMemoryStorage::new();
    repl::run(stdin, stdout, &mut storage).context("repl error")
}

/// Build a tokio runtime, bind, serve, drain on Ctrl-C.
fn run_server(addr: SocketAddr) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    runtime.block_on(async {
        let listener = TcpListener::bind(addr).await.context("failed to bind")?;
        eprintln!("kintoun listening on {addr}");
        let shutdown = async {
            let _ = tokio::signal::ctrl_c().await;
        };
        serve(
            listener,
            Arc::new(Mutex::new(InMemoryStorage::new())),
            shutdown,
        )
        .await?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an argv-like Vec<String> with the binary name in slot 0.
    fn args(extras: &[&str]) -> Vec<String> {
        std::iter::once("kintoun")
            .chain(extras.iter().copied())
            .map(String::from)
            .collect()
    }

    #[test]
    fn no_args_is_repl_mode() {
        assert_eq!(parse_args(args(&[])).unwrap(), Mode::Repl);
    }

    #[test]
    fn bind_with_addr_is_server_mode() {
        let parsed = parse_args(args(&["--bind", "127.0.0.1:4242"])).unwrap();
        let expected = Mode::Server("127.0.0.1:4242".parse().unwrap());
        assert_eq!(parsed, expected);
    }

    #[test]
    fn bind_without_addr_errors() {
        let err = parse_args(args(&["--bind"])).unwrap_err();
        assert!(
            err.to_string().contains("--bind requires an address"),
            "unexpected error: {err}",
        );
    }

    #[test]
    fn bind_with_invalid_addr_errors() {
        let err = parse_args(args(&["--bind", "not-an-addr"])).unwrap_err();
        assert!(
            err.to_string().contains("invalid socket address"),
            "unexpected error: {err}",
        );
    }

    #[test]
    fn extra_args_after_bind_error() {
        let err = parse_args(args(&["--bind", "127.0.0.1:4242", "extra"])).unwrap_err();
        assert!(
            err.to_string().contains("unexpected extra arguments"),
            "unexpected error: {err}",
        );
    }

    #[test]
    fn unknown_flag_errors() {
        let err = parse_args(args(&["--unknown"])).unwrap_err();
        assert!(
            err.to_string().contains("unknown argument"),
            "unexpected error: {err}",
        );
    }
}
