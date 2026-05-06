//! kintoun M2 server module.
//!
//! `serve()` runs the accept loop forever, spawning one
//! `handle_connection` task per accepted client. Caller owns the bind so
//! tests can use ephemeral ports and `main.rs` can use a configured
//! address.

pub mod codec;
pub mod connection;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::net::TcpListener;

use crate::storage::Storage;
use connection::handle_connection;

/// Drive the accept loop on `listener` forever.
///
/// Each accepted connection is dispatched on a fresh task via
/// `tokio::spawn`. Per-connection errors are swallowed — one bad client
/// must not tear the listener down. Returns `Err` only if the listener
/// itself fails (typically OS-level resource exhaustion).
///
/// `S: Send + 'static` because the spawned task captures
/// `Arc<Mutex<S>>` and outlives the `serve` call frame.
pub async fn serve<S>(listener: TcpListener, storage: Arc<Mutex<S>>) -> Result<()>
where
    S: Storage + Send + 'static,
{
    loop {
        let (socket, _peer) = listener.accept().await?;
        tokio::spawn(handle_connection(socket, storage.clone()));
    }
}
