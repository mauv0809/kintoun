//! kintoun-cli — interactive debug client for the M2 TCP server.
//!
//! Run: `cargo run --bin kintoun-cli -- --connect 127.0.0.1:4242`
//!
//! Each line of stdin becomes one length-prefixed frame on the wire;
//! the server's response payload is printed to stdout. Ctrl-D exits.
//! Reuses `FrameCodec` from the library crate so the client side
//! exercises the same codec the server does.

use std::net::SocketAddr;

use anyhow::{Context, Result, bail};
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use kintoun::server::codec::{Frame, FrameCodec};

/// Parse argv. Mirrors the grammar of `kintoun --bind`:
/// `kintoun-cli --connect <addr:port>` is the only valid form.
fn parse_args(args: Vec<String>) -> Result<SocketAddr> {
    let mut iter = args.into_iter().skip(1);
    match iter.next() {
        Some(arg) if arg == "--connect" => {
            let addr_str = iter
                .next()
                .context("--connect requires an address (e.g. 127.0.0.1:4242)")?;
            let addr: SocketAddr = addr_str
                .parse()
                .with_context(|| format!("invalid socket address: {addr_str}"))?;
            if iter.next().is_some() {
                bail!("unexpected extra arguments after --connect <addr>");
            }
            Ok(addr)
        }
        Some(other) => bail!("unknown argument: {other}"),
        None => bail!("usage: kintoun-cli --connect <addr:port>"),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let addr = parse_args(std::env::args().collect())?;

    let stream = TcpStream::connect(addr)
        .await
        .with_context(|| format!("failed to connect to {addr}"))?;
    eprintln!("connected to {addr}; ctrl-d to exit");

    let mut framed = Framed::new(stream, FrameCodec);
    let mut input = BufReader::new(tokio::io::stdin()).lines();

    while let Some(line) = input.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        framed
            .send(Frame::new(trimmed.as_bytes().to_vec()))
            .await
            .context("send failed")?;

        match framed.next().await {
            Some(Ok(frame)) => {
                println!("{}", String::from_utf8_lossy(&frame.payload));
            }
            Some(Err(e)) => bail!("codec error reading response: {e}"),
            None => {
                eprintln!("server closed connection");
                break;
            }
        }
    }

    eprintln!("bye");
    Ok(())
}
