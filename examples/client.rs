use futures::{SinkExt, StreamExt as _};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};
use zeevonk::packet::Packet;
use zeevonk::packet::client::ClientboundPacketPayload;
use zeevonk::packet::codec::{PacketDecoder, PacketEncoder};
use zeevonk::packet::server::ServerboundPacketPayload;

#[tokio::main]
async fn main() {
    let (reader, writer) = TcpStream::connect("127.0.0.1:7334").await.unwrap().into_split();
    let mut framed_reader =
        FramedRead::new(reader, PacketDecoder::<ClientboundPacketPayload>::default());
    let mut framed_writer = FramedWrite::new(writer, PacketEncoder::default());

    framed_writer.send(Packet::new(ServerboundPacketPayload::RequestDmxOutput)).await.unwrap();

    while let Some(packet) = framed_reader.next().await {
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
