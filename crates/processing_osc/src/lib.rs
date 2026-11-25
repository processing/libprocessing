//! OSC (Open Sound Control) over UDP
//!
//! `std::net::UdpSocket` sends and receives raw bytes;
//! rosc translates between a message and its raw bytes.
//! The sockets are stored in one Bevy resource [`OscState`]. 
//! Each `osc_*` function briefly accesses that resource while the sketch runs.

use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};

use bevy::prelude::*;
use rosc::{OscPacket, decoder, encoder};

use processing_core::app_mut;
use processing_core::error::{ProcessingError, Result};

pub use rosc::{OscArray, OscMessage, OscType};

pub struct OscPlugin;

impl Plugin for OscPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OscState>();
    }
}

/// A message received on the listening port, with the sender's address.
#[derive(Clone, Debug, PartialEq)]
pub struct ReceivedOscMessage {
    pub message: OscMessage,
    pub from: SocketAddr,
}

#[derive(Resource, Default)]
struct OscState {
    receiver: Option<UdpSocket>,
    /// Created on first send
    sender: Option<UdpSocket>,
}

/// Add the messages in `packet` to `out`, flattening nested bundles.
fn unfold(packet: OscPacket, from: SocketAddr, out: &mut Vec<ReceivedOscMessage>) {
    match packet {
        OscPacket::Message(message) => out.push(ReceivedOscMessage { message, from }),
        OscPacket::Bundle(bundle) => {
            for packet in bundle.content {
                unfold(packet, from, out);
            }
        }
    }
}

impl OscState {
    /// Bind the receive socket on all interfaces, replacing any previous one.
    /// Port 0 picks a free port. Returns the port actually bound.
    fn listen(&mut self, port: u16) -> Result<u16> {
        self.receiver = None;
        let failed = |e: std::io::Error| ProcessingError::OscListenFailed(port, e.to_string());
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port)).map_err(failed)?;
        // Reads happen once per frame and must never stall the sketch.
        socket.set_nonblocking(true).map_err(failed)?;
        let bound = socket.local_addr().map_err(failed)?.port();
        self.receiver = Some(socket);
        Ok(bound)
    }

    fn stop(&mut self) {
        self.receiver = None;
    }

    /// Drain every pending packet, unfolding bundles into their messages.
    fn read(&mut self) -> Result<Vec<ReceivedOscMessage>> {
        let socket = self
            .receiver
            .as_ref()
            .ok_or(ProcessingError::OscNotListening)?;
        let mut buf = [0u8; decoder::MTU];
        let mut received = Vec::new();
        loop {
            let (len, from) = match socket.recv_from(&mut buf) {
                Ok(r) => r,
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => return Err(ProcessingError::OscReceiveFailed(e.to_string())),
            };
            match decoder::decode_udp(&buf[..len]) {
                Ok((_, packet)) => unfold(packet, from, &mut received),
                // One bad packet shouldn't hide the ones queued behind it.
                Err(e) => warn!("Ignoring malformed OSC packet from {from}: {e:?}"),
            }
        }
        Ok(received)
    }

    /// Return receiver address
    fn send(&mut self, host: &str, port: u16, message: OscMessage) -> Result<SocketAddr> {
        let failed = |e: String| ProcessingError::OscSendFailed(e);
        let bytes =
            encoder::encode(&OscPacket::Message(message)).map_err(|e| failed(format!("{e:?}")))?;
        if self.sender.is_none() {
            let socket =
                UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(|e| failed(e.to_string()))?;
            self.sender = Some(socket);
        }
        let to = resolve_ipv4(host, port).map_err(failed)?;
        let socket = self.sender.as_ref().expect("sender was just created");
        socket
            .send_to(&bytes, to)
            .map_err(|e| failed(e.to_string()))?;
        Ok(to)
    }
}

/// Look up `host` and pick its IPv4 address
fn resolve_ipv4(host: &str, port: u16) -> std::result::Result<SocketAddr, String> {
    (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("could not find host {host:?}: {e}"))?
        .find(SocketAddr::is_ipv4)
        .ok_or_else(|| format!("host {host:?} has no IPv4 address; IPv6 isn't supported yet"))
}

/// A message as short readable text
pub fn osc_message_text(message: &OscMessage) -> String {
    let mut out = message.addr.clone();
    for arg in &message.args {
        out.push(' ');
        arg_text(arg, &mut out);
    }
    out
}

fn arg_text(value: &OscType, out: &mut String) {
    use std::fmt::Write;
    let _ = match value {
        OscType::Int(i) => write!(out, "{i}"),
        OscType::Long(i) => write!(out, "{i}"),
        OscType::Float(f) => write!(out, "{f}"),
        OscType::Double(d) => write!(out, "{d}"),
        OscType::String(s) => write!(out, "{s:?}"),
        OscType::Char(c) => write!(out, "{c:?}"),
        OscType::Blob(b) => write!(out, "<{} bytes>", b.len()),
        OscType::Bool(b) => write!(out, "{b}"),
        OscType::Nil => write!(out, "nil"),
        OscType::Inf => write!(out, "inf"),
        OscType::Time(t) => write!(out, "time({}, {})", t.seconds, t.fractional),
        OscType::Color(c) => write!(out, "rgba({}, {}, {}, {})", c.red, c.green, c.blue, c.alpha),
        OscType::Midi(m) => write!(
            out,
            "midi({}, {}, {}, {})",
            m.port, m.status, m.data1, m.data2
        ),
        OscType::Array(a) => {
            out.push('[');
            for (i, v) in a.content.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                arg_text(v, out);
            }
            out.push(']');
            Ok(())
        }
    };
}

fn with_state<T>(f: impl FnOnce(&mut OscState) -> Result<T>) -> Result<T> {
    app_mut(|app| f(&mut app.world_mut().resource_mut::<OscState>()))
}

/// Start listening for OSC messages on `port` (all network interfaces).
/// Calling it again switches to the new port. Returns the bound port, which
/// is useful when passing 0 to let the system choose.
#[cfg(not(target_arch = "wasm32"))]
pub fn osc_listen(port: u16) -> Result<u16> {
    with_state(|state| state.listen(port))
}

/// Stop listening and close the receive socket.
#[cfg(not(target_arch = "wasm32"))]
pub fn osc_stop() -> Result<()> {
    with_state(|state| {
        state.stop();
        Ok(())
    })
}

/// Every message received since the last call, oldest first. 
/// Messages inside bundles are returned individually.
#[cfg(not(target_arch = "wasm32"))]
pub fn osc_read() -> Result<Vec<ReceivedOscMessage>> {
    with_state(|state| state.read())
}

/// Send one message, e.g. `osc_send("127.0.0.1", 9000, "/xy", vec![OscType::Float(0.5)])`.
/// Returns the receiver address, with `host` looked up (e.g. `127.0.0.1:9000`).
#[cfg(not(target_arch = "wasm32"))]
pub fn osc_send(host: &str, port: u16, address: &str, args: Vec<OscType>) -> Result<SocketAddr> {
    let message = OscMessage {
        addr: address.to_string(),
        args,
    };
    with_state(|state| state.send(host, port, message))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rosc::{OscBundle, OscTime};
    use std::time::{Duration, Instant};

    /// Loopback delivery is fast but not instant, so poll briefly.
    fn read_until(state: &mut OscState, count: usize) -> Vec<ReceivedOscMessage> {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut got = Vec::new();
        while got.len() < count && Instant::now() < deadline {
            got.extend(state.read().unwrap());
            std::thread::sleep(Duration::from_millis(5));
        }
        got
    }

    fn msg(addr: &str, args: Vec<OscType>) -> OscMessage {
        OscMessage {
            addr: addr.into(),
            args,
        }
    }

    #[test]
    fn formats_messages_as_text() {
        let args = vec![
            OscType::Int(1),
            OscType::Float(0.52),
            OscType::String("hi".into()),
            OscType::Bool(true),
            OscType::Nil,
            OscType::Blob(vec![1, 2]),
            OscType::Array(OscArray {
                content: vec![OscType::Int(1), OscType::Double(2.5)],
            }),
        ];
        assert_eq!(
            osc_message_text(&msg("/mix", args)),
            r#"/mix 1 0.52 "hi" true nil <2 bytes> [1 2.5]"#
        );
        assert_eq!(osc_message_text(&msg("/empty", vec![])), "/empty");
    }

    #[test]
    fn localhost_resolves_to_ipv4() {
        let addr = resolve_ipv4("localhost", 9000).unwrap();
        assert_eq!(addr, "127.0.0.1:9000".parse().unwrap());
        assert!(resolve_ipv4("::1", 9000).is_err());
    }

    #[test]
    fn sends_and_receives_over_loopback() {
        let mut state = OscState::default();
        let port = state.listen(0).unwrap();

        let args = vec![
            OscType::Int(1),
            OscType::Float(0.5),
            OscType::String("hi".into()),
        ];
        state
            .send("127.0.0.1", port, msg("/test", args.clone()))
            .unwrap();

        let got = read_until(&mut state, 1);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].message.addr, "/test");
        assert_eq!(got[0].message.args, args);
        assert!(got[0].from.ip().is_loopback());
    }

    #[test]
    fn read_with_nothing_pending_returns_immediately() {
        let mut state = OscState::default();
        state.listen(0).unwrap();
        assert!(state.read().unwrap().is_empty());
    }

    #[test]
    fn bundles_are_flattened() {
        let from: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        let bundle = OscPacket::Bundle(OscBundle {
            timetag: OscTime::from((0, 1)),
            content: vec![
                OscPacket::Message(msg("/a", vec![])),
                OscPacket::Bundle(OscBundle {
                    timetag: OscTime::from((0, 1)),
                    content: vec![OscPacket::Message(msg("/b", vec![]))],
                }),
            ],
        });
        let mut out = Vec::new();
        unfold(bundle, from, &mut out);
        let addrs: Vec<_> = out.iter().map(|r| r.message.addr.as_str()).collect();
        assert_eq!(addrs, ["/a", "/b"]);
    }

    #[test]
    fn read_without_listening_is_an_error() {
        let mut state = OscState::default();
        assert!(matches!(
            state.read(),
            Err(ProcessingError::OscNotListening)
        ));
    }

    #[test]
    fn stop_closes_the_receiver() {
        let mut state = OscState::default();
        state.listen(0).unwrap();
        state.stop();
        assert!(matches!(
            state.read(),
            Err(ProcessingError::OscNotListening)
        ));
    }
}
