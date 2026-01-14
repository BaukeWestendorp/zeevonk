use std::cell::RefCell;
use std::net::IpAddr;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::server::output::sacn;
use crate::server::output::usb::enttec_open_dmx;
use crate::server::showfile::{Output, SacnMode};
use crate::server::{self, State};

// 44 hz
const DMX_OUTPUT_FRAME_TIME: Duration = Duration::from_micros(22_727);

// FIXME: We should find a way to create a unique UUID for a device, without it
// changing over it's lifetime.
const SACN_CID: sacn::ComponentIdentifier = sacn::ComponentIdentifier::from_bytes([
    0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
]);

pub fn start(output: Output, server_state: Arc<State>) {
    thread::Builder::new()
        .name("output".to_string())
        .spawn(move || {
            OutputAgent::new(output, server_state)
                .expect("should create new output process")
                .start();
        })
        .unwrap();
}

pub struct OutputAgent {
    server_state: Arc<State>,
    tx: crossbeam_channel::Sender<()>,
    rx: crossbeam_channel::Receiver<()>,
    sacn_sources: RefCell<Vec<JoinHandle<()>>>,
    enttec_open_dmx_devices: RefCell<Vec<JoinHandle<()>>>,
    shutdown: RefCell<bool>,
}

impl OutputAgent {
    pub fn new(output: Output, server_state: Arc<State>) -> Result<Self, server::Error> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let this = Self {
            server_state,
            tx,
            rx,
            sacn_sources: RefCell::new(Vec::new()),
            enttec_open_dmx_devices: RefCell::new(Vec::new()),
            shutdown: RefCell::new(false),
        };

        for sacn_output in output.sacn() {
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

        for enttec_open_dmx in output.usb().enttec_open_dmx() {
            this.add_enttec_open_dmx_device(enttec_open_dmx.serial_number())?;
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

            self.tx.send(()).expect("should send new frame notifier to output");

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

        // Join all threads
        for handle in self.enttec_open_dmx_devices.borrow_mut().drain(..) {
            let _ = handle.join();
        }
    }

    fn add_sacn_source(
        &self,
        name: String,
        ip: IpAddr,
        priority: u8,
        preview_data: bool,
    ) -> Result<(), server::Error> {
        let source = sacn::Source::new(sacn::SourceConfig {
            cid: SACN_CID,
            name,
            ip,
            port: sacn::DEFAULT_PORT,
            priority,
            preview_data,
            synchronization_address: 0,
            force_synchronization: false,
        })?;

        self.spawn_sacn_source_thread(source);

        Ok(())
    }

    fn add_enttec_open_dmx_device(&self, serial_number: &str) -> Result<(), server::Error> {
        let interface = enttec_open_dmx::Interface::new(serial_number).unwrap();

        self.spawn_enttec_open_dmx_device_thread(interface);

        Ok(())
    }

    fn spawn_sacn_source_thread(&self, source: sacn::Source) {
        let rx = self.rx.clone();
        let server_state = self.server_state.clone();
        let handle = thread::spawn(move || {
            while let Ok(()) = rx.recv() {
                let multiverse = server_state.output_multiverse.blocking_read().clone();
                for (id, universe) in multiverse.universes() {
                    let mut sacn_universe = sacn::Universe::new(id.as_u16());
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

    fn spawn_enttec_open_dmx_device_thread(&self, mut interface: enttec_open_dmx::Interface) {
        let rx = self.rx.clone();
        let server_state = self.server_state.clone();
        let handle = thread::spawn(move || {
            interface.open().unwrap();
            while let Ok(()) = rx.recv() {
                let multiverse = server_state.output_multiverse.blocking_read().clone();
                for (_, universe) in multiverse.universes() {
                    interface
                        .write_universe(universe.clone())
                        .map_err(|err| log::error!("failed to send universe data over sACN: {err}"))
                        .ok();
                }
            }
            interface.close().unwrap();
        });

        self.enttec_open_dmx_devices.borrow_mut().push(handle);
    }
}

impl Drop for OutputAgent {
    fn drop(&mut self) {
        self.shutdown();
    }
}
