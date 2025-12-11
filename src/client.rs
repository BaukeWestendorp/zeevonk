use std::sync::Arc;
use std::time::Duration;

use futures::{SinkExt, StreamExt as _};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio::{io, task};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::dmx::Multiverse;
use crate::packet::{
    ClientboundPacketPayload, Packet, PacketDecoder, PacketEncoder, ServerboundPacketPayload,
};
use crate::server::{AttributeValues, BakedPatch};
use crate::util::TimingLogger;

/// The Zeevonk client.
pub struct Client {
    inner: Arc<Mutex<Inner>>,
}

impl Client {
    /// Connects to a Zeevonk server at the given address.
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let (reader, writer) = TcpStream::connect(addr).await?.into_split();
        let decoder = PacketDecoder::<ClientboundPacketPayload>::default();
        let encoder = PacketEncoder::default();
        let packet_reader = FramedRead::new(reader, decoder);
        let packet_writer = FramedWrite::new(writer, encoder);

        let inner = Arc::new(Mutex::new(Inner { packet_reader, packet_writer }));

        Ok(Self { inner })
    }

    /// Registers a processor closure that will run in a background task.
    ///
    /// The processor is invoked on a fixed 25ms interval (i.e. 40Hz).
    ///
    /// The populated attribute values are sent to the server on each frame.
    pub async fn register_processor<
        F: Fn(usize, &BakedPatch, &mut AttributeValues) + Send + Sync + 'static,
    >(
        &self,
        processor: F,
    ) {
        let inner = Arc::clone(&self.inner);
        let processor = Arc::new(processor);
        task::spawn(async move {
            let baked_patch = match inner.lock().await.request_patch().await {
                Ok(p) => p,
                Err(err) => {
                    log::error!("could not get baked patch for processor: {err}");
                    return;
                }
            };

            // Use a fixed interval starting one period from now to get accurate 25ms ticks.
            let period = Duration::from_millis(25);
            let start = tokio::time::Instant::now() + period;
            let mut interval = tokio::time::interval_at(start, period);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            // For averaging logs over time.
            const AVG_WINDOW: usize = 80;
            let mut timing_logger = TimingLogger::new(AVG_WINDOW);

            let mut i = 0;
            loop {
                // Wait until the next scheduled tick. Using interval_at fixes the schedule
                // to the chosen start instant and period, minimizing drift.
                let scheduled_instant = interval.tick().await;
                let start_instant = tokio::time::Instant::now();

                // How late we are relative to the scheduled instant.
                let lateness =
                    start_instant.checked_duration_since(scheduled_instant).unwrap_or_default();

                let mut values = AttributeValues::new();

                let proc_start = tokio::time::Instant::now();
                (processor.as_ref())(i, &baked_patch, &mut values);
                let proc_end = tokio::time::Instant::now();

                let send_start = tokio::time::Instant::now();
                // Await the result to ensure the request is sent and handled.
                let send_result = inner.lock().await.request_set_attribute_values(values).await;
                let send_end = tokio::time::Instant::now();

                let proc_duration = proc_end.duration_since(proc_start);
                let send_duration = send_end.duration_since(send_start);
                let total_frame = send_end.duration_since(start_instant);

                // Record and possibly log timings
                timing_logger.record_frame(
                    lateness,
                    proc_duration,
                    send_duration,
                    total_frame,
                    period,
                );

                if let Err(err) = send_result {
                    log::error!("failed to send attribute values: {err}");
                    break;
                }

                i += 1;
            }
        })
        .await
        .unwrap();
    }

    /// Requests the currently baked patch from the server.
    pub async fn request_patch(&self) -> io::Result<BakedPatch> {
        let mut guard = self.inner.lock().await;
        guard.request_patch().await
    }

    /// Requests the current DMX output (multiverse) from the server.
    pub async fn request_dmx_output(&self) -> io::Result<Multiverse> {
        let mut guard = self.inner.lock().await;
        guard.request_dmx_output().await
    }

    /// Requests setting attribute values for fixtures on the server.
    pub async fn request_set_attribute_values(&self, values: AttributeValues) -> io::Result<()> {
        let mut guard = self.inner.lock().await;
        guard.request_set_attribute_values(values).await
    }
}

struct Inner {
    packet_reader: FramedRead<OwnedReadHalf, PacketDecoder<ClientboundPacketPayload>>,
    packet_writer: FramedWrite<OwnedWriteHalf, PacketEncoder<ServerboundPacketPayload>>,
}

impl Inner {
    /// Requests the currently baked patch from the server.
    pub async fn request_patch(&mut self) -> io::Result<BakedPatch> {
        self.send_packet(ServerboundPacketPayload::RequestBakedPatch).await?;

        while let Some(packet) = self.packet_reader.next().await {
            match packet {
                Ok(packet) => match packet.payload {
                    ClientboundPacketPayload::ResponseBakedPatch(baked_patch) => {
                        return Ok(baked_patch);
                    }
                    _ => continue,
                },
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
    }

    /// Requests the current DMX output (multiverse) from the server.
    pub async fn request_dmx_output(&mut self) -> io::Result<Multiverse> {
        self.send_packet(ServerboundPacketPayload::RequestDmxOutput).await?;

        while let Some(packet) = self.packet_reader.next().await {
            match packet {
                Ok(packet) => match packet.payload {
                    ClientboundPacketPayload::ResponseDmxOutput(multiverse) => {
                        return Ok(multiverse);
                    }
                    _ => continue,
                },
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
    }

    /// Requests setting attribute values for fixtures on the server.
    pub async fn request_set_attribute_values(
        &mut self,
        values: AttributeValues,
    ) -> io::Result<()> {
        self.send_packet(ServerboundPacketPayload::RequestSetAttributeValues { values }).await?;

        while let Some(packet) = self.packet_reader.next().await {
            match packet {
                Ok(packet) => match packet.payload {
                    ClientboundPacketPayload::ResponseSetAttributeValues => {
                        return Ok(());
                    }
                    _ => continue,
                },
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
    }

    /// Sends a packet with the given payload to the server.
    async fn send_packet(&mut self, payload: ServerboundPacketPayload) -> io::Result<()> {
        self.packet_writer
            .send(Packet::new(payload))
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
