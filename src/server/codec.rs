//! Frame codec for the kintoun M2 wire protocol.
//!
//! Wire format (per ADR 0012):
//!     [len:u32 BE][payload bytes]
//!
//! Payload is opaque at this layer — `connection.rs` interprets it as
//! UTF-8 command text and feeds it into `cmd::parse`.

use bytes::{Buf, BufMut, BytesMut};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

/// Maximum allowed frame payload size, in bytes.
/// Bounds memory usage from malicious or malformed clients.
/// (Per ADR 0012; revisit if benchmarks suggest otherwise.)
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024; // 16 MiB

/// One frame on the wire. Payload is opaque bytes; the codec doesn't
/// inspect them. `connection.rs` decodes payload-as-text after framing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub payload: Vec<u8>,
}
impl Frame {
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }
}

/// Errors produced by the frame codec.
///
/// `Io` carries the underlying `std::io::Error` (raised by tokio when
/// the socket itself misbehaves). `FrameTooLarge` is our policy check.
#[derive(Debug, Error)]
pub enum CodecError {
    #[error("frame size {size} exceeds max {max}")]
    FrameTooLarge { size: usize, max: usize },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Stateless frame codec.
///
/// `Decoder::decode` is called repeatedly by tokio's `Framed` adapter
/// as bytes accumulate in the buffer. It must return:
///   - `Ok(Some(frame))` when one complete frame is available
///   - `Ok(None)` when more bytes are needed
///   - `Err(_)` on protocol violation (e.g., oversize frame)
pub struct FrameCodec;

impl Decoder for FrameCodec {
    type Item = Frame;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Frame>, CodecError> {
        if src.len() < 4 {
            return Ok(None);
        }
        let len = u32::from_be_bytes(
            src[..4]
                .try_into()
                .expect("BUG: src.len() >= 4 verified above"),
        );
        if len > MAX_FRAME_SIZE as u32 {
            return Err(CodecError::FrameTooLarge {
                size: len as usize,
                max: MAX_FRAME_SIZE,
            });
        }
        if src.len() < 4 + len as usize {
            return Ok(None);
        }
        src.advance(4);
        let payload = src.split_to(len as usize).to_vec();
        Ok(Some(Frame { payload }))
    }
}

impl Encoder<Frame> for FrameCodec {
    type Error = CodecError;

    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), CodecError> {
        let payload = item.payload;
        if payload.len() > MAX_FRAME_SIZE {
            return Err(CodecError::FrameTooLarge {
                size: payload.len(),
                max: MAX_FRAME_SIZE,
            });
        }
        dst.reserve(4 + payload.len());
        dst.put_u32(payload.len() as u32);
        dst.extend_from_slice(&payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trip() {
        let mut codec = FrameCodec;
        let original_frame = Frame::new(b"SET foo bar".to_vec());

        // Encode into a fresh buffer
        let mut buf = BytesMut::new();
        codec
            .encode(original_frame.clone(), &mut buf)
            .expect("encode should succeed");

        // Decode from the same buffer
        let decoded = codec.decode(&mut buf).expect("decode should not error");

        assert_eq!(decoded, Some(original_frame));
        assert!(
            buf.is_empty(),
            "buffer should be drained after decoding the only frame"
        );
    }

    #[test]
    fn oversize_decode_returns_error() {
        let mut codec = FrameCodec;
        let mut buf = BytesMut::new();

        // Build a prefix that declares one byte more than the cap.
        // Cast to u32 explicitly so to_be_bytes() returns exactly 4 bytes.
        let oversized: u32 = (MAX_FRAME_SIZE as u32) + 1;
        buf.extend_from_slice(&oversized.to_be_bytes());

        let result = codec.decode(&mut buf);
        match result {
            Err(CodecError::FrameTooLarge { size, max }) => {
                assert_eq!(size, oversized as usize);
                assert_eq!(max, MAX_FRAME_SIZE);
            }
            other => panic!("expected FrameTooLarge, got {other:?}"),
        }
    }

    #[test]
    fn oversize_encode_returns_error() {
        let mut codec = FrameCodec;
        let oversize_frame = Frame::new(vec![0; MAX_FRAME_SIZE + 1]);
        let mut buf = BytesMut::new();

        let expected_size = oversize_frame.payload.len();
        let result = codec.encode(oversize_frame, &mut buf);
        match result {
            Err(CodecError::FrameTooLarge { size, max }) => {
                assert_eq!(size, expected_size);
                assert_eq!(max, MAX_FRAME_SIZE);
            }
            other => panic!("expected FrameTooLarge, got {other:?}"),
        }
    }
    #[test]
    fn empty_payload_returns_zero_length_frame() {
        let mut codec = FrameCodec;
        let empty_frame = Frame { payload: vec![] };
        let mut buf = BytesMut::new();
        codec
            .encode(empty_frame.clone(), &mut buf)
            .expect("encode should succeed");

        let result = codec.decode(&mut buf).expect("decode should not error");
        assert_eq!(result, Some(empty_frame));
    }
    #[test]
    fn partial_input_no_full_prefix() {
        let mut codec = FrameCodec;
        let mut buf = BytesMut::new();

        // Only 3 bytes — not even a full length prefix (need 4)
        buf.extend_from_slice(&[0, 0, 0]);

        let result = codec.decode(&mut buf).expect("decode should not error");
        assert_eq!(result, None);
        assert_eq!(buf.len(), 3, "buffer should be unchanged");
    }

    #[test]
    fn partial_input_prefix_only() {
        let mut codec = FrameCodec;
        let mut buf = BytesMut::new();

        // 4 bytes of prefix declaring 11 bytes of body, but body hasn't arrived
        buf.extend_from_slice(&[0, 0, 0, 11]);

        let result = codec.decode(&mut buf).expect("decode should not error");
        assert_eq!(result, None);
        assert_eq!(buf.len(), 4, "prefix should not be consumed");
    }

    #[test]
    fn partial_input_short_body() {
        let mut codec = FrameCodec;
        let mut buf = BytesMut::new();

        // Prefix says 11 bytes; only 5 body bytes sent so far
        buf.extend_from_slice(&[0, 0, 0, 11]);
        buf.extend_from_slice(b"SET f");

        let result = codec.decode(&mut buf).expect("decode should not error");
        assert_eq!(result, None);
        assert_eq!(buf.len(), 9, "buffer unchanged when body incomplete");
    }
    #[test]
    fn glued_frames_decode_twice() {
        // Encode two frames into the same buffer; decode should yield each in order.
        let mut codec = FrameCodec;
        let frame_a = Frame::new(b"GET foo".to_vec());
        let frame_b = Frame::new(b"SET bar baz".to_vec());
        let mut buf = BytesMut::new();
        codec.encode(frame_a.clone(), &mut buf).unwrap();
        codec.encode(frame_b.clone(), &mut buf).unwrap();

        let first = codec.decode(&mut buf).unwrap();
        assert_eq!(first, Some(frame_a));

        let second = codec.decode(&mut buf).unwrap();
        assert_eq!(second, Some(frame_b));

        assert!(buf.is_empty(), "both frames should be consumed");
    }

    #[test]
    fn incremental_decode_one_byte_at_a_time() {
        // Feed bytes one at a time; decode returns None until the last byte arrives.
        let mut codec = FrameCodec;
        let frame = Frame::new(b"INCR counter".to_vec());

        // First, build the wire bytes by encoding into a scratch buffer.
        let mut wire = BytesMut::new();
        codec.encode(frame.clone(), &mut wire).unwrap();
        let total = wire.len();

        // Now feed one byte at a time into a real buffer.
        let mut buf = BytesMut::new();
        for i in 0..total - 1 {
            buf.extend_from_slice(&wire[i..=i]);
            let result = codec.decode(&mut buf).unwrap();
            assert_eq!(result, None, "decode should need more bytes at i={i}");
        }

        // Last byte arrives — decode now returns the complete frame.
        buf.extend_from_slice(&wire[total - 1..total]);
        let result = codec.decode(&mut buf).unwrap();
        assert_eq!(result, Some(frame));
        assert!(buf.is_empty());
    }

    #[test]
    fn decode_preserves_extra_bytes() {
        // After decoding one frame, any leftover bytes from a partial second frame
        // must remain in the buffer (decoder doesn't eat what it can't decode).
        let mut codec = FrameCodec;
        let frame_a = Frame::new(b"OK".to_vec());
        let mut buf = BytesMut::new();
        codec.encode(frame_a.clone(), &mut buf).unwrap();

        // Append the start of a second frame: prefix says 5 bytes, only 2 arrive.
        buf.extend_from_slice(&[0, 0, 0, 5]);
        buf.extend_from_slice(b"hi");

        let first = codec.decode(&mut buf).unwrap();
        assert_eq!(first, Some(frame_a));

        // The 4-byte prefix + 2 partial body bytes of frame B should still be there.
        assert_eq!(buf.len(), 6);

        let second = codec.decode(&mut buf).unwrap();
        assert_eq!(second, None, "frame B is incomplete; needs 3 more bytes");
    }
    #[test]
    fn idempotent_decode_on_starvation() {
        let mut codec = FrameCodec;
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0, 0, 0, 5, b'h']); // prefix says 5; only 1 body byte

        for _ in 0..3 {
            let result = codec.decode(&mut buf).unwrap();
            assert_eq!(result, None);
            assert_eq!(buf.len(), 5, "buffer not consumed across repeated calls");
        }
    }
    #[test]
    fn binary_clean_payload_round_trips() {
        // Payload contains the bytes a text-only codec would mishandle:
        // null, line endings, high-bit byte, valid multi-byte UTF-8, and
        // an INVALID UTF-8 byte (codec must not care about UTF-8 validity).
        let mut codec = FrameCodec;
        let payload: Vec<u8> = vec![
            0x00, // NUL
            b'\n', b'\r', // line endings
            0xFF,  // high-bit non-ASCII
            0xE2, 0x9C, 0x93, // valid UTF-8 for ✓ (U+2713)
            0xC0, // invalid UTF-8 lead byte
        ];
        let frame = Frame::new(payload);

        let mut buf = BytesMut::new();
        codec.encode(frame.clone(), &mut buf).unwrap();
        let decoded = codec.decode(&mut buf).unwrap();

        assert_eq!(
            decoded,
            Some(frame),
            "codec must be byte-agnostic per ADR 0012"
        );
    }
    #[test]
    fn max_frame_size_boundary_round_trips() {
        let mut codec = FrameCodec;
        let frame = Frame::new(vec![0xAB; MAX_FRAME_SIZE]);
        let mut buf = BytesMut::new();

        codec
            .encode(frame.clone(), &mut buf)
            .expect("exact MAX_FRAME_SIZE should encode");
        assert_eq!(buf.len(), 4 + MAX_FRAME_SIZE);

        let decoded = codec
            .decode(&mut buf)
            .expect("exact MAX_FRAME_SIZE should decode");
        assert_eq!(decoded, Some(frame));
    }
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn arbitrary_payload_round_trips(
            payload in proptest::collection::vec(any::<u8>(), 0..=10_000),
        ) {
            let mut codec = FrameCodec;
            let frame = Frame::new(payload);
            let mut buf = BytesMut::new();
            codec.encode(frame.clone(), &mut buf).unwrap();
            let decoded = codec.decode(&mut buf).unwrap();
            prop_assert_eq!(decoded, Some(frame));
        }
    }
}
