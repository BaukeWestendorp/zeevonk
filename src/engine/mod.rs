use crate::engine::output::DmxOutputManager;
use crate::showfile::Showfile;

mod output;

pub struct Engine {
    dmx_output_manager: DmxOutputManager,
}

impl Engine {
    pub fn new(showfile: Showfile) -> Self {
        Self { dmx_output_manager: DmxOutputManager::new(showfile.protocols()) }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.dmx_output_manager.start();

        loop {}
    }
}
