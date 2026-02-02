use theymx::Multiverse;

use crate::{output, project::file::dmx_output::DmxOutputInstanceDefinition};

pub mod enttec_open_dmx;
pub mod sacn;

pub trait OutputInstanceImplementation {
    fn setup(&mut self) -> Result<(), output::Error>;

    fn handle_frame(&mut self, frame: Multiverse) -> Result<(), output::Error>;

    fn shutdown(&mut self) -> Result<(), output::Error>;
}

pub struct OutputInstance {
    implementation: Box<dyn OutputInstanceImplementation>,
}

impl OutputInstance {
    pub fn new<I: OutputInstanceImplementation + 'static>(implementation: I) -> Self {
        Self { implementation: Box::new(implementation) }
    }

    pub fn run(
        &mut self,
        output_rx: crossbeam_channel::Receiver<Multiverse>,
    ) -> Result<(), output::Error> {
        self.implementation.setup()?;

        while let Ok(frame) = output_rx.recv() {
            self.implementation.handle_frame(frame)?;
        }

        self.implementation.shutdown()?;

        Ok(())
    }
}

impl TryFrom<DmxOutputInstanceDefinition> for OutputInstance {
    type Error = output::Error;

    fn try_from(definition: DmxOutputInstanceDefinition) -> Result<Self, Self::Error> {
        let instance = match definition {
            DmxOutputInstanceDefinition::EnttecOpenDmx { universe_id, serial_number } => {
                let eod = enttec_open_dmx::EnttecOpenDmxOutput::new(universe_id, serial_number)?;
                Self::new(eod)
            }
            DmxOutputInstanceDefinition::Sacn {
                name,
                universe_ids,
                preview_mode,
                priority,
                target_address,
            } => {
                let sacn = sacn::SacnOutput::new(
                    name,
                    universe_ids,
                    preview_mode,
                    priority,
                    target_address,
                )?;
                Self::new(sacn)
            }
        };

        Ok(instance)
    }
}
