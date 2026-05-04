# ADR 0012: M2 Wire Protocol — TCP with Length-Prefixed Envelope and Text Payload

Date: 2026-05-04
Status: Accepted (implementation pending in `src/server/`)

## Context

M1 closes with kintoun reachable only via stdin/stdout REPL. M2 exposes the same verbs to remote clients. The wire-format choice is foundational: once a client speaks it, changing it is a coordinated upgrade. It also cascades through the milestone arc — M4's pub/sub push frames, M6's peer-replication messages, and M7's Raft RPCs all reuse the same envelope, only the payload changes.

Three pressures shaped the decision:

1. **Pedagogical fit.** kintoun is a learn-Rust vehicle. The M2 lesson is "framed protocol over a byte stream + tokio I/O traits." Whatever protocol we ship has to teach that, not bury it.
2. **Future-proofing across the arc.** M4 push frames must not force a refactor of M2. M6 peer protocol and M7 Raft RPCs must reuse the same envelope.
3. **The function-color split.** Tokio's async I/O traits (`AsyncRead`, `AsyncWrite`) are different from the sync ones (`Read`, `Write`); Rust has no clean abstraction over both. Whatever we pick lives in the async world cleanly or it doesn't fit at all.

TCP is byte-stream, not message-oriented: `read(buf)` may return half a message, two messages glued together, or any other split. The first job of any TCP protocol is inventing message boundaries — framing.

### Alternatives Considered

**A — Newline-delimited text (REPL-on-a-wire).** Telnet-debuggable; reuses M1 parser unchanged; trivial codec. Rejected because M4 push frames force a refactor: a server-initiated `NOTIFY foo bar\n` is indistinguishable from a response, requiring a magic prefix or out-of-band signaling. Avoiding known-doomed decisions is a project rule.

**B1 — Length-prefix envelope with text payload (chosen).** `[len:u32 BE][UTF-8 command line]`. Two-state codec: read 4 bytes of length, read N body bytes. Body is the same text M1's `cmd::parse` already accepts. The envelope is the new lesson; the payload reuses what works.

**B2 — Length-prefix envelope with binary opcodes.** `[len:u32 BE][op:u8][argc:u8][arg_len:u32][arg]…`. Closer to the user's prior BLE protocol; more compact on the wire. Rejected for M2 specifically: payload format is decoupled from envelope by design, so the opcode lesson can land at M3 (WAL records) where binary serialization is a natural fit and the rule for "what's the canonical wire shape" lives next to the data.

**C — RESP-like (Redis Serialization Protocol).** Self-describing typed format; production-grade; what Redis itself uses. Rejected: more codec states to track (off-by-one bugs on `\r\n` boundaries are a classic source); RESP-specific learning that doesn't transfer to Raft wire format at M7; production interop with the Redis ecosystem isn't a project goal worth the complexity.

**gRPC, HTTP/REST.** Rejected as M2 substrates: hide the wire-format learning that's the project's core. Can be added later as parallel client-facing surfaces (see Deferred Surfaces in `memory.md`'s milestone arc); their reasoning belongs in their own ADR when that milestone lands.

**Per-frame payload-format byte in the envelope.** Considered explicitly. Rejected: conflates frame-type (response / push / peer) with payload-format (text / binary / protobuf). Real protocols (HTTP/2, MQTT, gRPC) negotiate format once at handshake or fix it protocol-wide; per-frame format negotiation is a use case that almost never materializes. YAGNI.

**Unix domain sockets (UDS) instead of TCP.** Mechanically identical in tokio (`UnixListener` mirrors `TcpListener`); same async cliff; same framing problem. Rejected: useless for M6 cross-host replication.

## Decision

1. **Transport: TCP.** Default bind `127.0.0.1:4242`. `--bind <addr:port>` override (see ADR 0013 for CLI handling).

2. **Envelope: `[len:u32 BE][payload]`.** Four-byte big-endian length prefix; raw payload bytes; no other fields in M2. Frame-size cap enforced server-side (default 16MB, configurable) to bound denial-of-service exposure from malicious or malformed clients.

3. **Payload: UTF-8 text command line.** M1's `cmd::parse` runs unchanged on the body bytes. The body is the same string a user would type into the REPL — no escaping, no framing characters inside (since we have explicit length).

4. **Errors to client: `ERR <message>` payload.** Sent as a regular frame. Clients distinguish `OK`, value responses, and errors by inspecting the first token of the payload — same convention as REPL stdout per ADR 0010.

5. **No type bytes in the M2 envelope.** Frame-type byte is deferred until M4, when push frames first need disambiguation from responses.

## Consequences

- **The M1 parser is reused unchanged.** `cmd::parse` is the canonical command parser across REPL and TCP. One source of truth; one set of tests.
- **The codec maps directly to `tokio_util::codec::Decoder`/`Encoder`.** This is the canonical async-Rust I/O abstraction. The skill generalizes to M4 (push frames), M6 (peer messages), M7 (Raft RPCs) — all reuse the envelope, only the payload changes.
- **Payload format is opaque to the envelope.** Future milestones can swap to binary opcodes, MessagePack, protobuf, or a tagged binary format without touching framing. The envelope/payload split is the load-bearing design property.
- **Binary-clean by construction.** Keys and values can contain any bytes — `\0`, `\n`, `\r`, multi-byte UTF-8 — because length is authoritative. The text-payload constraint at M2 is a parser convention, not a wire constraint.
- **Push frames at M4 add a single byte.** `[len:u32 BE][frame_type:u8][payload]`. One-time wire-format break, executed under our exclusive control of client and server. Reserve byte ranges in the M4 ADR: `0x00`=command, `0x01`=response, `0x02`=push, `0x10–0x1F`=peer (M6), `0x20–0x2F`=consensus (M7).
- **Not telnet-debuggable.** A tiny debug client (~30 lines of Python or a small Rust binary) is required from day one. Telnet won't prepend the four-byte length.
- **Hand-rolled codec.** ~30 lines of Rust plus tests. The hand-roll is the lesson; we're not pulling in a dependency to skip it.
- **Frame-size cap is on us.** A `u32` length permits 4GB frames in principle; the configured cap (default 16MB) bounds memory exhaustion attacks. Without a cap a single malicious client could allocate arbitrarily.

## Open Follow-ups

- **Frame-size cap value.** Default 16MB feels right; revisit if benchmarks or workload patterns suggest otherwise.
- **Debug-client tool.** Decide language (Python one-shot vs. a crate-internal `kintoun-cli` binary) when implementation starts. Pythonic one-shot is sufficient for M2 testing; a real CLI tool is a stretch milestone in its own right.
- **Environment-variable bind override (`KINTOUN_BIND`).** In addition to `--bind`. Cosmetic; defer until telemetry/deployment use cases call for it.
- **Frame-type byte allocation at M4.** Pre-allocate the byte ranges in the M4 ADR rather than scattering them across multiple ADRs. This ADR reserves the *concept*; M4 binds the values.
- **gRPC as a parallel client-facing surface.** Captured in `memory.md` Deferred Surfaces. When the client-SDK stretch milestone lands, write an ADR covering the parallel-surface decision: gRPC and TCP both speak to the same `executor`/`storage`, neither replaces the other.
