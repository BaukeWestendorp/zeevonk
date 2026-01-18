use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::{io, thread};

use tungstenite::Message;

use crate::packet::processor::ServerboundPacket;
use crate::server::output::agent::OutputAgent;
use crate::server::{self};

pub struct ProcessorListener {
    output_agent: Arc<OutputAgent>,
}

impl ProcessorListener {
    pub fn new(output_agent: Arc<OutputAgent>) -> Self {
        Self { output_agent }
    }

    pub fn start<A: ToSocketAddrs>(&mut self, address: A) -> Result<(), server::Error> {
        let listener = TcpListener::bind(address)?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let output_agent = Arc::clone(&self.output_agent);
                    thread::spawn(move || {
                        if let Err(err) = accept_stream(stream, output_agent) {
                            log::error!("processor client handler failed: {err}");
                        }
                    });
                }
                Err(err) => {
                    log::error!("processor stream closed: {err}");
                    return Err(err.into());
                }
            }
        }

        Ok(())
    }
}

fn accept_stream(stream: TcpStream, output_agent: Arc<OutputAgent>) -> crate::Result<()> {
    let peer_addr = stream.peer_addr().unwrap();

    log::info!("connected to processor at {}", peer_addr);

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

                if let Err(e) = handle_packet(packet, &output_agent) {
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

    log::info!("connection with processor at {} closed", peer_addr);

    Ok(())
}

fn handle_packet(packet: ServerboundPacket, output_agent: &OutputAgent) -> crate::Result<()> {
    match packet {
        ServerboundPacket::RegisterClient { .. } => todo!(),
        ServerboundPacket::UpdateAttributes { values, include_children } => {
            if include_children {
                todo!();
            }

            output_agent.update_values(values);
        }
    }

    Ok(())
}
