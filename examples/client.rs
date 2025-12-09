use std::io::{ErrorKind, Read as _, Write};
use std::net::TcpStream;

use zeevonk::packet::{ClientboundPacket, ServerboundPacket};

pub fn main() {
    let mut socket = TcpStream::connect("127.0.0.1:7334").unwrap();

    socket.write(&ServerboundPacket::RequestDmxOutput.encode_packet_bytes()).unwrap();

    loop {
        // Read the first 4 bytes for the length
        let mut len_buf = [0u8; 4];
        let mut read_len = 0;
        while read_len < 4 {
            match socket.read(&mut len_buf[read_len..]) {
                Ok(0) => return, // connection closed
                Ok(n) => read_len += n,
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    eprintln!("error reading length from socket: {}", e);
                    return;
                }
            }
        }
        let packet_len = u32::from_le_bytes(len_buf) as usize;

        // Read the payload
        let mut payload = vec![0u8; packet_len];
        let mut read_payload = 0;
        while read_payload < packet_len {
            match socket.read(&mut payload[read_payload..]) {
                Ok(0) => return, // connection closed
                Ok(n) => read_payload += n,
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    eprintln!("error reading payload from socket: {}", e);
                    return;
                }
            }
        }

        let packet = ClientboundPacket::decode_payload_bytes(&payload).unwrap();

        dbg!(packet);
    }
}
