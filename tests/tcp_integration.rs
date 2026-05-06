//! Real-TCP integration tests for the kintoun M2 server.
//!
//! Lives in `tests/` (a separate crate from the library) so the same
//! wire path runs end-to-end:
//!     bind → accept → handle_connection → codec → executor → response
//!
//! Helpers mirror the duplex-pipe tests in `src/server/connection.rs`
//! but go through a real `TcpStream`. If a third integration-test file
//! ever appears, lift these into `tests/common/mod.rs`.

use std::sync::{Arc, Mutex};

use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

use kintoun::server::codec::{Frame, FrameCodec, MAX_FRAME_SIZE};
use kintoun::server::serve;
use kintoun::storage::InMemoryStorage;

// =========================================================================
// Helpers
// =========================================================================

/// Bind an ephemeral port, spawn `serve`, return the address clients dial.
///
/// The server task is detached — there's no JoinHandle to await; the
/// listener runs until the test process exits. Tokio cleans up at runtime
/// shutdown.
async fn spawn_server() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let storage = Arc::new(Mutex::new(InMemoryStorage::new()));
    tokio::spawn(serve(listener, storage));
    addr
}

/// Connect a client to `addr` and wrap in a `Framed` over `FrameCodec`.
async fn connect(addr: std::net::SocketAddr) -> Framed<TcpStream, FrameCodec> {
    let sock = TcpStream::connect(addr).await.unwrap();
    Framed::new(sock, FrameCodec)
}

/// Send one frame on the client side. Same shape as the duplex helper.
async fn send_request(client: &mut Framed<TcpStream, FrameCodec>, payload: &[u8]) {
    client
        .send(Frame::new(payload.to_vec()))
        .await
        .expect("send should succeed");
}

/// Receive one frame's payload from the server.
async fn recv_response(client: &mut Framed<TcpStream, FrameCodec>) -> Vec<u8> {
    match client.next().await {
        Some(Ok(frame)) => frame.payload,
        Some(Err(e)) => panic!("codec error reading response: {e}"),
        None => panic!("server closed before sending response"),
    }
}

// =========================================================================
// Tests
// =========================================================================

#[tokio::test]
async fn set_then_get_round_trips_over_real_tcp() {
    let addr = spawn_server().await;
    let mut client = connect(addr).await;

    send_request(&mut client, b"SET foo bar").await;
    let response = recv_response(&mut client).await;
    assert_eq!(response, b"OK");

    send_request(&mut client, b"GET foo").await;
    let response = recv_response(&mut client).await;
    assert_eq!(response, b"\"bar\"");
}

#[tokio::test]
async fn two_clients_share_storage_via_real_accept() {
    let addr = spawn_server().await;
    let mut client_one = connect(addr).await;
    let mut client_two = connect(addr).await;

    send_request(&mut client_one, b"SET foo bar").await;
    let response = recv_response(&mut client_one).await;
    assert_eq!(response, b"OK");

    send_request(&mut client_two, b"GET foo").await;
    let response = recv_response(&mut client_two).await;
    assert_eq!(response, b"\"bar\"");
}

#[tokio::test]
async fn oversize_frame_closes_connection() {
    let addr = spawn_server().await;
    let mut socket = TcpStream::connect(addr).await.unwrap();
    let oversize = (MAX_FRAME_SIZE as u32) + 1;
    let prefix = oversize.to_be_bytes();
    socket.write_all(&prefix).await.unwrap();
    let mut buf = Vec::new();
    socket.read_to_end(&mut buf).await.unwrap();
    assert!(
        buf.is_empty(),
        "expected empty buffer; server wrote {buf:?}"
    );
}

#[tokio::test]
async fn partial_prefix_then_close_is_clean_eof() {
    // Hostile-input variant of test 3, but exercising a different server-side
    // code path: the codec's `decode_eof` rather than `decode`-returns-error.
    //
    // Client writes 2 of 4 length-prefix bytes, then half-closes (FIN). The
    // server's `Framed` reads the 2 bytes (codec returns Ok(None) — needs
    // more), then reads EOF. The default `decode_eof` impl returns an io
    // error ("bytes remaining on stream"); `handle_connection` propagates
    // Err; the spawned task drops its TcpStream. From the client's read
    // half: EOF, no bytes written.
    //
    // We use `shutdown()` instead of `drop(socket)` so the *write* half
    // closes (FIN) but the *read* half stays open to observe the server's
    // response (which should be: nothing).
    let addr = spawn_server().await;
    let mut socket = TcpStream::connect(addr).await.unwrap();

    socket.write_all(&[0x00, 0x00]).await.unwrap();
    socket.shutdown().await.unwrap();

    let mut buf = Vec::new();
    socket.read_to_end(&mut buf).await.unwrap();
    assert!(
        buf.is_empty(),
        "expected empty buffer; server wrote {buf:?}",
    );
}

#[tokio::test]
async fn pipelined_commands_preserve_order() {
    // Send N frames back-to-back without reading responses between them,
    // then drain N responses. Asserts:
    //   1. The codec correctly handles glued frames over a real socket
    //      (multiple frames living in one read chunk).
    //   2. Responses come back in send-order on a single connection.
    //
    // Functional richness lives at the duplex layer; this is the wire-level
    // smoke test for pipelining specifically.
    let addr = spawn_server().await;
    let mut client = connect(addr).await;

    send_request(&mut client, b"SET a 1").await;
    send_request(&mut client, b"SET b 2").await;
    send_request(&mut client, b"SET c 3").await;

    assert_eq!(recv_response(&mut client).await, b"OK");
    assert_eq!(recv_response(&mut client).await, b"OK");
    assert_eq!(recv_response(&mut client).await, b"OK");

    // Confirm all three landed AND that response order matched send order.
    // Values were numeric, so the executor's `from_text` inference stored
    // them as Int; format emits them bare (no quotes), unlike the Str case.
    send_request(&mut client, b"GET a").await;
    assert_eq!(recv_response(&mut client).await, b"1");
    send_request(&mut client, b"GET b").await;
    assert_eq!(recv_response(&mut client).await, b"2");
    send_request(&mut client, b"GET c").await;
    assert_eq!(recv_response(&mut client).await, b"3");
}

#[tokio::test]
async fn concurrent_incr_serializes_via_mutex() {
    // Two clients each INCR a counter K times concurrently. The Mutex inside
    // Arc<Mutex<Storage>> must serialize the read-modify-writes; if it
    // didn't, lost updates would push the final value below 2K.
    //
    // Caveat: `#[tokio::test]` defaults to a current-thread runtime, so the
    // spawned tasks share one OS thread. This test exercises cooperative-
    // yield concurrency, not true parallelism. A stress-grade version would
    // set `flavor = "multi_thread"`. For M2, the cooperative case already
    // proves the per-connection serialization invariant: the SET/INCR
    // critical section never observes a partial state from a peer.
    const ITERATIONS: usize = 100;

    let addr = spawn_server().await;

    // Initialize the counter from a setup connection.
    let mut setup = connect(addr).await;
    send_request(&mut setup, b"SET foo 0").await;
    assert_eq!(recv_response(&mut setup).await, b"OK");

    let task_a = tokio::spawn(async move {
        let mut client = connect(addr).await;
        for _ in 0..ITERATIONS {
            send_request(&mut client, b"INCR foo").await;
            let _ = recv_response(&mut client).await;
        }
    });
    let task_b = tokio::spawn(async move {
        let mut client = connect(addr).await;
        for _ in 0..ITERATIONS {
            send_request(&mut client, b"INCR foo").await;
            let _ = recv_response(&mut client).await;
        }
    });
    task_a.await.expect("task a panicked");
    task_b.await.expect("task b panicked");

    // Verify the final counter from a fresh connection.
    let mut verifier = connect(addr).await;
    send_request(&mut verifier, b"GET foo").await;
    let response = recv_response(&mut verifier).await;
    let expected = (2 * ITERATIONS).to_string();
    assert_eq!(
        response,
        expected.as_bytes(),
        "expected {expected} (= 2 * {ITERATIONS}); got {response:?}",
    );
}
