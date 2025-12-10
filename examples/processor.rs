use futures::{SinkExt as _, StreamExt as _};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio_util::codec::{FramedRead, FramedWrite};

use zeevonk::engine::Layout;
use zeevonk::gdcs::Attribute;
use zeevonk::packet::{
    ClientboundPacketPayload, Packet, PacketDecoder, PacketEncoder, ServerboundPacketPayload,
};

#[tokio::main]
async fn main() {
    let (r, w) = TcpStream::connect("127.0.0.1:7334").await.unwrap().into_split();
    let mut reader = FramedRead::new(r, PacketDecoder::<ClientboundPacketPayload>::default());
    let mut writer = FramedWrite::new(w, PacketEncoder::default());

    writer.send(Packet::new(ServerboundPacketPayload::RequestDmxOutput)).await.unwrap();
    writer.send(Packet::new(ServerboundPacketPayload::RequestLayout)).await.unwrap();

    while let Some(packet) = reader.next().await {
        match packet {
            Ok(packet) => match packet.payload() {
                ClientboundPacketPayload::ResponseLayout(layout) => {
                    process_layout(layout, &mut writer).await
                }
                ClientboundPacketPayload::ResponseSetAttributeValues => {
                    println!("attribute values have been set")
                }
                ClientboundPacketPayload::ResponseDmxOutput(multiverse) => {
                    println!("multiverse: {multiverse:?}")
                }
                _ => {}
            },
            Err(err) => {
                log::error!("error reading packet: {}", err);
                break;
            }
        }
    }

    async fn process_layout(
        layout: &Layout,
        framed_writer: &mut FramedWrite<OwnedWriteHalf, PacketEncoder>,
    ) {
        let mut values = Vec::new();

        for fixture in layout.fixtures() {
            let dimmer_channel_functions = fixture
                .channel_functions()
                .into_iter()
                .filter(|(attr, _cf)| **attr == Attribute::Dimmer);

            for (attr, cf) in dimmer_channel_functions {
                values.push((fixture.path(), attr.clone(), cf.to()));
            }
        }

        framed_writer
            .send(Packet::new(ServerboundPacketPayload::RequestSetAttributeValues { values }))
            .await
            .unwrap();

        framed_writer.send(Packet::new(ServerboundPacketPayload::RequestDmxOutput)).await.unwrap();
    }
}
