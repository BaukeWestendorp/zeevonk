use std::sync::{Arc, RwLock, mpsc};
use std::thread::{self};
use std::time::Duration;

use anyhow::Context as _;
use uuid::Uuid;

use crate::dmx::Multiverse;
use crate::server::sacn;
use crate::showfile::protocols::{Protocols, SacnMode};

const DMX_OUTPUT_INTERVAL: Duration = Duration::from_millis(25);

pub struct DmxOutputManager {
    sacn_sources: Vec<SacnSource>,
    output_multiverse: Arc<RwLock<Multiverse>>,
}

impl DmxOutputManager {
    /// Creates a new `DmxOutputManager` with the given protocols and output multiverse.
    pub fn new(
        protocols: &Protocols,
        output_multiverse: Arc<RwLock<Multiverse>>,
    ) -> anyhow::Result<Self> {
        // FIXME: This CID should be unique for the device, but the same over different showfiles.
        let cid = Uuid::default();

        let sacn_sources = protocols
            .sacn()
            .outputs()
            .iter()
            .map(|output| -> anyhow::Result<_> {
                let ip = match output.mode() {
                    SacnMode::Unicast { destination_ip } => destination_ip,
                    SacnMode::Multicast => todo!("implement sACN multicasting"),
                };

                let source = sacn::Source::new(sacn::SourceConfig {
                    cid,
                    name: output.label().to_string(),
                    ip,
                    port: sacn::DEFAULT_PORT,
                    priority: output.priority(),
                    ..Default::default()
                })?;

                let (data_tx, data_rx) = mpsc::channel();

                Ok(SacnSource {
                    local_universes: output.local_universes().to_owned(),
                    destination_universe: output.destination_universe(),
                    thread_handle: None,
                    data_tx,
                    data_rx: Some(data_rx),
                    source: Arc::new(source),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { sacn_sources, output_multiverse })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        for (ix, source) in self.sacn_sources.iter_mut().enumerate() {
            log::debug!("sACN source {ix} started");
            source.start();
        }

        log::debug!("all sACN sources started");

        loop {
            if let Err(err) = self.send_multiverse(&self.output_multiverse.read().unwrap()) {
                log::error!("failed to send multiverse: {err}");
            }

            spin_sleep::sleep(DMX_OUTPUT_INTERVAL);
        }
    }

    pub fn stop(&mut self) {
        let sources = std::mem::take(&mut self.sacn_sources);

        for mut source in sources {
            // Explicitly drop the sender so receivers see EOF.
            drop(source.data_tx);
            // If the thread was started, join it to ensure it has stopped.
            if let Some(handle) = source.thread_handle.take() {
                if let Err(err) = handle.join() {
                    log::error!("sACN source thread panicked while joining: {:?}", err);
                }
            }
        }
    }

    pub fn send_multiverse(&self, multiverse: &Multiverse) -> anyhow::Result<()> {
        for (id, universe) in multiverse.universes() {
            let Some(sacn_source) =
                self.sacn_sources.iter().find(|source| source.local_universes.contains(id))
            else {
                continue;
            };

            let mut sacn_universe = sacn::Universe::new(sacn_source.destination_universe);
            sacn_universe.data_slots = universe.values().iter().map(|v| v.0).collect();
            sacn_source
                .send_universe_data_packet(sacn_universe)
                .map_err(|err| log::error!("failed to send universe data over sACN: {err}"))
                .ok();
        }

        Ok(())
    }
}

impl Drop for DmxOutputManager {
    fn drop(&mut self) {
        self.stop();
    }
}

struct SacnSource {
    pub local_universes: Vec<u16>,
    pub destination_universe: u16,
    pub thread_handle: Option<thread::JoinHandle<()>>,
    pub data_tx: mpsc::Sender<sacn::Universe>,
    // The value will be taken out of `data_rx` (leaving `None`) when the thread is started,
    // as the thread takes ownership of the receiver.
    pub data_rx: Option<mpsc::Receiver<sacn::Universe>>,
    pub source: Arc<sacn::Source>,
}

impl SacnSource {
    pub fn start(&mut self) {
        let thread_handle = thread::spawn({
            let Some(data_rx) = self.data_rx.take() else { return };
            let source = Arc::clone(&self.source);
            move || {
                while let Ok(universe) = data_rx.recv() {
                    source
                        .send_universe_data_packet(universe)
                        .map_err(|err| log::error!("failed to send universe data over sACN: {err}"))
                        .ok();
                }
            }
        });
        self.thread_handle = Some(thread_handle);
    }

    pub fn send_universe_data_packet(&self, universe: sacn::Universe) -> anyhow::Result<()> {
        self.data_tx
            .send(universe)
            .context("universe data channel is closed (has it been opened by starting the source?")
    }
}
