use theymx::Multiverse;

use crate::project::OutputInstanceDefinition;

pub mod enttec_open_dmx;

pub trait OutputInstanceImplementation {
    fn setup(&mut self) -> Result<(), crate::output::Error>;

    fn handle_frame(&mut self, frame: Multiverse) -> Result<(), crate::output::Error>;

    fn shutdown(&mut self) -> Result<(), crate::output::Error>;
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
    ) -> Result<(), crate::output::Error> {
        self.implementation.setup()?;

        while let Ok(frame) = output_rx.recv() {
            self.implementation.handle_frame(frame)?;
        }

        self.implementation.shutdown()?;

        Ok(())
    }
}

impl TryFrom<OutputInstanceDefinition> for OutputInstance {
    type Error = crate::output::Error;

    fn try_from(definition: OutputInstanceDefinition) -> Result<Self, Self::Error> {
        let instance = match definition {
            OutputInstanceDefinition::EnttecOpenDmx { universe_id, serial_number } => {
                let eod = enttec_open_dmx::EnttecOpenDmxOutput::new(universe_id, serial_number)?;
                Self::new(eod)
            }
        };

        Ok(instance)
    }
}
