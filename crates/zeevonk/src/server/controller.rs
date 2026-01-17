use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::{io, thread};

use tungstenite::Message;

use crate::packet::controller::ServerboundPacket;
use crate::server::{self};

pub struct ControllerListener {}

impl ControllerListener {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start<A: ToSocketAddrs>(&mut self, address: A) -> Result<(), server::Error> {
        let listener = TcpListener::bind(address)?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        if let Err(err) = accept_stream(stream) {
                            log::error!("controller client handler failed: {err}");
                        }
                    });
                }
                Err(err) => {
                    log::error!("controller stream closed: {err}");
                    return Err(err.into());
                }
            }
        }

        Ok(())
    }
}

fn accept_stream(stream: TcpStream) -> crate::Result<()> {
    let peer_addr = stream.peer_addr().unwrap();

    log::info!("connected to controller at {}", peer_addr);

    let mut socket = match tungstenite::accept(stream) {
        Ok(ws) => ws,
        Err(err) => {
            log::error!("failed to accept socket connection: {err}");
            return Err(crate::Error::Io(io::Error::other(err)));
        }
    };

    loop {
        let message = match socket.read() {
            Ok(message) => message,
            Err(tungstenite::Error::ConnectionClosed) => break,
            Err(err) => {
                log::error!("failed to read socket message: {err}");
                return Err(crate::Error::Io(io::Error::other(err)));
            }
        };

        match message {
            Message::Close(_) => return Ok(()),
            Message::Text(json) => {
                let packet = match serde_json::from_str(json.as_str()) {
                    Ok(packet) => packet,
                    Err(e) => {
                        log::error!("failed to parse ServerboundPacket: {e}");
                        continue;
                    }
                };

                if let Err(e) = handle_packet(packet) {
                    log::error!("error handling packet: {e}");
                }
            }
            _ => {}
        }
    }

    if let Err(err) = socket.close(None) {
        log::error!("failed to close socket: {err}");
        return Err(crate::Error::Io(io::Error::other(err)));
    }

    log::info!("connection with controller at {} closed", peer_addr);

    Ok(())
}

fn handle_packet(packet: ServerboundPacket) -> crate::Result<()> {
    match packet {
        ServerboundPacket::Trigger { trigger } => {
            log::info!("received trigger: {trigger:?}");
        }
    }

    Ok(())
}
