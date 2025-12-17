use std::cell::RefCell;
use std::net::IpAddr;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::Error;
use crate::server::ServerState;
use crate::server::protocols::sacn;
use crate::showfile::{Protocols, SacnMode};

const DMX_OUTPUT_FRAME_TIME: Duration = Duration::from_millis(44);

// FIXME: We should find a way to create a unique UUID for a device, without it
// changing over it's lifetime.
const SACN_CID: sacn::ComponentIdentifier = sacn::ComponentIdentifier::from_bytes([
    0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
]);

pub fn start(protocols: Protocols, server_state: Arc<ServerState>) {
    thread::Builder::new()
        .name("protocols".to_string())
        .spawn(move || {
            ProtocolsProcess::new(protocols, server_state)
                .expect("should create new protocols process")
                .start();
        })
        .unwrap();
}

pub struct ProtocolsProcess {
    server_state: Arc<ServerState>,
    tx: crossbeam_channel::Sender<()>,
    rx: crossbeam_channel::Receiver<()>,
    sacn_sources: RefCell<Vec<JoinHandle<()>>>,
    shutdown: RefCell<bool>,
}

impl ProtocolsProcess {
    pub fn new(protocols: Protocols, server_state: Arc<ServerState>) -> Result<Self, Error> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let this = Self {
            server_state,
            tx,
            rx,
            sacn_sources: RefCell::new(Vec::new()),
            shutdown: RefCell::new(false),
        };

        for sacn_output in protocols.sacn().outputs() {
            let ip = match sacn_output.mode() {
                SacnMode::Unicast { destination_ip } => destination_ip,
                SacnMode::Multicast => todo!(),
            };

            this.add_sacn_source(
                sacn_output.label().to_owned(),
                ip,
                sacn_output.priority(),
                sacn_output.preview_data(),
            )?;
        }

        Ok(this)
    }

    pub fn start(self) {
        let start_time = Instant::now();
        let mut frame_count = 0;
        let mut total_frame_time = Duration::ZERO;

        loop {
            let frame_start = Instant::now();

            let target_time = start_time + DMX_OUTPUT_FRAME_TIME * frame_count;
            let now = Instant::now();

            if frame_count != 0 {
                if now < target_time {
                    spin_sleep::sleep(target_time - now);
                } else {
                    let overrun = now - target_time;
                    if overrun > DMX_OUTPUT_FRAME_TIME {
                        log::warn!("frame {frame_count} overrun by {overrun:?}");
                    }
                }
            }

            self.tx.send(()).expect("should send new frame notifier to protocols");

            let frame_end = Instant::now();
            let frame_time = frame_end - frame_start;
            total_frame_time += frame_time;

            log::trace!("frame {frame_count} time: {frame_time:?}");

            frame_count += 1;
        }
    }

    pub fn shutdown(&self) {
        let mut shutdown = self.shutdown.borrow_mut();
        if *shutdown {
            return;
        }
        *shutdown = true;

        // Join all threads
        for handle in self.sacn_sources.borrow_mut().drain(..) {
            let _ = handle.join();
        }
    }

    fn add_sacn_source(
        &self,
        name: String,
        ip: IpAddr,
        priority: u8,
        preview_data: bool,
    ) -> Result<(), Error> {
        let source = sacn::Source::new(sacn::SourceConfig {
            cid: SACN_CID,
            name,
            ip,
            port: sacn::DEFAULT_PORT,
            priority,
            preview_data,
            synchronization_address: 0,
            force_synchronization: false,
        })
        .map_err(|err| Error::Server { message: err.to_string() })?;

        self.spawn_sacn_source_thread(source);

        Ok(())
    }

    fn spawn_sacn_source_thread(&self, source: sacn::Source) {
        let rx = self.rx.clone();
        let server_state = self.server_state.clone();
        let handle = thread::spawn(move || {
            while let Ok(()) = rx.recv() {
                let multiverse = server_state.output_multiverse.blocking_read().clone();
                for (id, universe) in multiverse.universes() {
                    let mut sacn_universe = sacn::Universe::new(**id);
                    sacn_universe.data_slots = universe.values().iter().map(|v| v.0).collect();
                    source
                        .send_universe_data_packet(sacn_universe)
                        .map_err(|err| log::error!("failed to send universe data over sACN: {err}"))
                        .ok();
                }
            }
        });

        self.sacn_sources.borrow_mut().push(handle);
    }
}

impl Drop for ProtocolsProcess {
    fn drop(&mut self) {
        self.shutdown();
    }
}
