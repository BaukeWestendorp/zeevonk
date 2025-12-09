use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};

use crate::dmx::Multiverse;
use crate::showfile::protocols::Protocols;

/// How often the DMX output should be sent to the protocols.
const DMX_OUTPUT_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);

/// Manages DMX output threads and protocol outputs.
pub struct DmxOutputManager {
    /// List of sACN outputs.
    _sacn_outputs: Vec<SacnOutput>,

    /// Handle to the thread.
    thread_handle: Option<JoinHandle<()>>,

    /// Shared reference to the output multiverse.
    _output_multiverse: Arc<RwLock<Multiverse>>,
}

impl DmxOutputManager {
    /// Creates a new `DmxOutputManager` with the given protocols and output multiverse.
    pub fn new(protocols: &Protocols, _output_multiverse: Arc<RwLock<Multiverse>>) -> Self {
        let _sacn_outputs = protocols.sacn().outputs().iter().map(|_| SacnOutput {}).collect();
        Self { _sacn_outputs, thread_handle: None, _output_multiverse }
    }

    /// Starts the DMX output manager thread.
    pub fn start(&mut self) {
        log::info!("starting dmx output manager");

        let thread_handle = thread::spawn(start_thread);
        self.thread_handle = Some(thread_handle);
    }
}

/// The main DMX output thread loop.
fn start_thread() {
    loop {
        log::trace!("sending dmx output...");
        send_dmx_output();
        log::trace!("dmx output sent");
        spin_sleep::sleep(DMX_OUTPUT_INTERVAL);
    }
}

/// Sends DMX output to all configured outputs.
fn send_dmx_output() {}

/// Represents a single sACN output.
struct SacnOutput {}
