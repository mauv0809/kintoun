//! kintoun M2 server module.
//!
//! `serve()` runs the accept loop until the caller-provided shutdown
//! future resolves. Per-connection tasks are tracked in a `JoinSet`
//! so the function can drain them gracefully before returning.
//! Caller owns the bind so tests can use ephemeral ports and `main.rs`
//! can use a configured address.

pub mod codec;
pub mod connection;

use std::future::Future;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::net::TcpListener;
use tokio::task::JoinSet;

use crate::storage::Storage;
use connection::handle_connection;

/// Drive the accept loop on `listener` until `shutdown` resolves.
///
/// Each accepted connection is dispatched into a `JoinSet` so the loop
/// can later drain in-flight work. Per-connection errors are swallowed
/// (during accept and during drain) — one bad client must not tear the
/// listener down, nor block shutdown.
///
/// On `shutdown`:
///   1. The accept loop exits (no new connections accepted).
///   2. The listener is dropped (the OS releases the port).
///   3. The JoinSet is drained — `serve` waits for every in-flight task
///      to finish naturally (i.e. when its client closes the connection).
///      No timeout at M2; tracked in ADR 0013 as an open follow-up.
///
/// Returns `Err` only if the listener itself fails (typically OS-level
/// resource exhaustion via `accept()`); shutdown returns `Ok(())`.
///
/// Test rigs pass `std::future::pending::<()>()` for `shutdown` to get
/// "serve forever" behaviour. The binary passes `tokio::signal::ctrl_c()`
/// (wrapped to `Future<Output = ()>`).
///
/// `S: Send + 'static` because spawned `handle_connection` tasks capture
/// `Arc<Mutex<S>>` and may run on any worker thread of a multi-thread
/// runtime. `F` has no `Send` bound — the shutdown future stays pinned
/// on the serve task's stack and is never moved across threads.
pub async fn serve<S, F>(listener: TcpListener, storage: Arc<Mutex<S>>, shutdown: F) -> Result<()>
where
    S: Storage + Send + 'static,
    F: Future<Output = ()>,
{
    let mut tasks: JoinSet<Result<()>> = JoinSet::new();

    // `select!` needs a pinned future to poll on each branch evaluation
    // without consuming it. `F` isn't `Unpin` in general, so pin it on
    // the stack.
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
        accept = listener.accept() => {
            let (sock, _peer) = accept?;
            tasks.spawn(handle_connection(sock, storage.clone()));
        }
        _ = &mut shutdown => break,
        }
    }
    drop(listener);
    while tasks.join_next().await.is_some() {}
    Ok(())
}
