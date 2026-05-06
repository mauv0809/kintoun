//! Per-connection async task body for the kintoun TCP server.
//!
//! Lifecycle (one task per accepted connection):
//!   1. Wrap the raw stream in `Framed` with `FrameCodec`.
//!   2. Loop: read frame → decode UTF-8 → parse → execute → format → send.
//!   3. End on clean EOF (Ok) or fatal error (Err).
//!
//! Error policy (locked in M2 design):
//!   - codec / utf-8 / write errors → close connection (propagate via `?`)
//!   - parse / executor errors      → send `ERR <msg>` and continue loop
//!
//! Lock discipline (per ADR 0013 + Tokio tutorial ch. 3):
//!   `std::sync::Mutex` is acquired in a *block scope* so the guard drops
//!   before any `.await`. The compiler enforces this — `MutexGuard` is
//!   `!Send`, so holding it across `.await` would make the future `!Send`
//!   and `tokio::spawn` would refuse to compile it.

use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt}; // .next() / .send() on Framed
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

use crate::cmd;
use crate::executor;
use crate::format;
use crate::server::codec::{Frame, FrameCodec};
use crate::storage::Storage;

/// Drive one client connection to completion.
///
/// Generic over the I/O type so tests can pass `tokio::io::duplex` halves
/// instead of a real `TcpStream`. Generic over `Storage` so the same body
/// works against any backend (M3+ WAL-backed will inherit).
///
/// The `Send` bound on `S` is needed because the future captures
/// `Arc<Mutex<S>>`, and `Mutex<S>: Sync` only when `S: Send`.
pub async fn handle_connection<IO, S>(stream: IO, storage: Arc<Mutex<S>>) -> Result<()>
where
    IO: AsyncRead + AsyncWrite + Unpin,
    S: Storage + Send,
{
    let mut framed = Framed::new(stream, FrameCodec);

    // .next() yields one frame at a time. None = stream closed cleanly.
    while let Some(frame_result) = framed.next().await {
        // Codec error here = protocol violation. `?` closes the connection.
        let frame: Frame = frame_result.context("codec error reading frame")?;

        // Build the response string. This block deliberately does NOT use `?`
        // for parse/executor errors — those become ERR responses and the
        // loop continues. Only fatal errors propagate out.
        let response: String = match std::str::from_utf8(&frame.payload) {
            // UTF-8 violation → close. (Per error policy table.)
            Err(e) => return Err(anyhow::anyhow!("invalid utf-8 in payload: {e}")),

            Ok(line) => process_line(line, &storage),
        };

        // Send response as a Frame. Write error → propagate (can't talk back).
        let out = Frame::new(response.into_bytes());
        framed
            .send(out)
            .await
            .context("write error sending response")?;
    }

    // Loop exit via `None` = clean EOF from client. Treat as Ok.
    Ok(())
}

/// Decode → parse → execute → format. Errors return `ERR <msg>` strings
/// rather than propagating; only the outer loop owns the connection.
///
/// Pulled out into a sync helper because the whole pipeline is sync —
/// no awaits live in here. Keeps the lock-discipline obvious: the
/// `MutexGuard` exists only inside the inner block.
fn process_line<S: Storage>(line: &str, storage: &Arc<Mutex<S>>) -> String {
    let command = match cmd::Command::from_str(line) {
        Ok(c) => c,
        Err(e) => return format::format_error(&e),
    };

    // === The lock-discipline block ===
    // Guard's lifetime ends at the closing brace. No .await inside here.
    // .unwrap() on lock() is the standard pattern: the only way it fails
    // is poisoning (panic in another holder), and we don't recover from
    // that — let it propagate as a panic.
    let exec_result = {
        let mut store = storage.lock().unwrap();
        executor::execute(&mut *store, command)
        // guard dropped here ↑↑↑
    };

    match exec_result {
        Ok(r) => format::format_result(&r),
        Err(e) => format::format_error(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStorage;

    // --- helpers ---
    async fn recv_response<IO>(client: &mut Framed<IO, FrameCodec>) -> Vec<u8>
    where
        IO: AsyncRead + AsyncWrite + Unpin,
    {
        match client.next().await {
            Some(Ok(frame)) => frame.payload,
            Some(Err(e)) => panic!("codec error reading response: {e}"),
            None => panic!("server closed before sending response"),
        }
    }

    async fn send_request<IO>(client: &mut Framed<IO, FrameCodec>, payload: &[u8])
    where
        IO: AsyncRead + AsyncWrite + Unpin,
    {
        match client.send(Frame::new(payload.to_vec())).await {
            Ok(_) => {}
            Err(e) => panic!("write error sending request: {e}"),
        }
    }

    #[tokio::test]
    async fn set_then_get_round_trips_over_duplex() {
        let (server_io, client_io) = tokio::io::duplex(1024);
        let storage = Arc::new(Mutex::new(InMemoryStorage::new()));

        let _server = tokio::spawn(handle_connection(server_io, storage));
        let mut client = Framed::new(client_io, FrameCodec);

        send_request(&mut client, b"SET foo bar").await;
        let response = recv_response(&mut client).await;
        assert_eq!(response, b"OK");

        send_request(&mut client, b"GET foo").await;
        let response = recv_response(&mut client).await;
        assert_eq!(response, b"\"bar\"");
    }

    #[tokio::test]
    async fn parse_error_sends_err_and_keeps_connection_alive() {
        let (server_io, client_io) = tokio::io::duplex(1024);
        let storage = Arc::new(Mutex::new(InMemoryStorage::new()));

        let _server = tokio::spawn(handle_connection(server_io, storage));
        let mut client = Framed::new(client_io, FrameCodec);

        send_request(&mut client, b"asdjasiodjasd").await;
        let response = recv_response(&mut client).await;
        assert!(response.starts_with(b"ERR "));

        send_request(&mut client, b"SET foo bar").await;
        let response = recv_response(&mut client).await;
        assert_eq!(response, b"OK");
    }

    #[tokio::test]
    async fn clean_eof_completes_with_ok() {
        let (server_io, client_io) = tokio::io::duplex(1024);
        let storage = Arc::new(Mutex::new(InMemoryStorage::new()));

        let server = tokio::spawn(handle_connection(server_io, storage));
        let client = Framed::new(client_io, FrameCodec);

        drop(client);

        let join_result = server.await.expect("task panicked");
        assert!(
            join_result.is_ok(),
            "handle_connection returned: {join_result:?}"
        );
    }

    #[tokio::test]
    async fn executor_error_sends_err_and_keeps_connection_alive() {
        // Symmetric to parse_error_*: the ERR comes from the executor branch
        // (StorageError::NotAnInteger), not from cmd::parse. Same continuation rule.
        let (server_io, client_io) = tokio::io::duplex(1024);
        let storage = Arc::new(Mutex::new(InMemoryStorage::new()));

        let _server = tokio::spawn(handle_connection(server_io, storage));
        let mut client = Framed::new(client_io, FrameCodec);

        // Stash a string, then attempt INCR — executor returns NotAnInteger.
        send_request(&mut client, b"SET foo hello").await;
        assert_eq!(recv_response(&mut client).await, b"OK");

        send_request(&mut client, b"INCR foo").await;
        let response = recv_response(&mut client).await;
        assert!(
            response.starts_with(b"ERR "),
            "expected ERR-prefixed response, got: {response:?}"
        );

        // Connection still alive — overwrite with a valid value.
        send_request(&mut client, b"SET foo bar").await;
        assert_eq!(recv_response(&mut client).await, b"OK");
    }

    #[tokio::test]
    async fn utf8_violation_closes_connection() {
        // 0xC0 is an invalid UTF-8 lead byte. The codec is byte-agnostic, so
        // the frame arrives intact; handle_connection's str::from_utf8 fails
        // and the function returns Err per the error policy.
        let (server_io, client_io) = tokio::io::duplex(1024);
        let storage = Arc::new(Mutex::new(InMemoryStorage::new()));

        let server = tokio::spawn(handle_connection(server_io, storage));
        let mut client = Framed::new(client_io, FrameCodec);

        send_request(&mut client, &[0xC0]).await;

        let join_result = server.await.expect("task panicked");
        assert!(
            join_result.is_err(),
            "expected Err on UTF-8 violation, got: {join_result:?}"
        );
    }

    #[tokio::test]
    async fn multiple_connections_share_storage() {
        // Two independent connection tasks, one Arc<Mutex<Storage>>: a write
        // from connection A must be visible to a read from connection B.
        // Locks the shared-state design end-to-end.
        let storage = Arc::new(Mutex::new(InMemoryStorage::new()));

        let (server_a, client_a_io) = tokio::io::duplex(1024);
        let _conn_a = tokio::spawn(handle_connection(server_a, Arc::clone(&storage)));
        let mut client_a = Framed::new(client_a_io, FrameCodec);

        let (server_b, client_b_io) = tokio::io::duplex(1024);
        let _conn_b = tokio::spawn(handle_connection(server_b, Arc::clone(&storage)));
        let mut client_b = Framed::new(client_b_io, FrameCodec);

        send_request(&mut client_a, b"SET foo bar").await;
        assert_eq!(recv_response(&mut client_a).await, b"OK");

        send_request(&mut client_b, b"GET foo").await;
        assert_eq!(recv_response(&mut client_b).await, b"\"bar\"");
    }

    #[tokio::test]
    async fn commands_evolve_state_in_sequence() {
        // Sanity check that storage state actually accumulates across frames.
        // Set numeric, increment, read back the resulting value.
        let (server_io, client_io) = tokio::io::duplex(1024);
        let storage = Arc::new(Mutex::new(InMemoryStorage::new()));

        let _server = tokio::spawn(handle_connection(server_io, storage));
        let mut client = Framed::new(client_io, FrameCodec);

        send_request(&mut client, b"SET counter 5").await;
        assert_eq!(recv_response(&mut client).await, b"OK");

        send_request(&mut client, b"INCR counter 3").await;
        assert_eq!(recv_response(&mut client).await, b"8");

        send_request(&mut client, b"GET counter").await;
        assert_eq!(recv_response(&mut client).await, b"8");
    }
}
