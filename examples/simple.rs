use futures::{SinkExt, StreamExt as _};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};

use zeevonk::packet::{
    ClientboundPacketPayload, Packet, PacketDecoder, PacketEncoder, ServerboundPacketPayload,
};

#[tokio::main]
async fn main() {
    let (r, w) = TcpStream::connect("127.0.0.1:7334").await.unwrap().into_split();
    let mut reader = FramedRead::new(r, PacketDecoder::<ClientboundPacketPayload>::default());
    let mut writer = FramedWrite::new(w, PacketEncoder::default());

    writer.send(Packet::new(ServerboundPacketPayload::RequestDmxOutput)).await.unwrap();

    while let Some(packet) = reader.next().await {
        match packet {
            Ok(packet) => {
                eprintln!("packet: {packet:?}");
            }
            Err(err) => {
                log::error!("error reading packet: {}", err);
                break;
            }
        }
    }
}
