use std::thread::{self, JoinHandle};

use crate::showfile::protocols::Protocols;

const DMX_OUTPUT_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);

pub struct DmxOutputManager {
    sacn_outputs: Vec<SacnOutput>,

    thread_handle: Option<JoinHandle<()>>,
}

impl DmxOutputManager {
    pub fn new(protocols: &Protocols) -> Self {
        let sacn_outputs = protocols.sacn().outputs().iter().map(|_| SacnOutput {}).collect();
        Self { sacn_outputs, thread_handle: None }
    }

    pub fn start(&mut self) {
        log::info!("starting dmx output manager");

        let thread_handle = thread::spawn(start_thread);
        self.thread_handle = Some(thread_handle);
    }
}

fn start_thread() {
    loop {
        log::trace!("sending dmx output...");
        send_dmx_output();
        log::trace!("dmx output sent");
        spin_sleep::sleep(DMX_OUTPUT_INTERVAL);
    }
}

fn send_dmx_output() {
    log::warn!("implement sending dmx output...");
}

struct SacnOutput {}
