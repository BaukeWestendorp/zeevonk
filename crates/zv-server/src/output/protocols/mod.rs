use theymx::Multiverse;

use crate::project::OutputInstanceDefinition;

pub mod enttec_open_dmx;

pub trait OutputInstanceImplementation {
    fn setup(&mut self);

    fn handle_frame(&mut self, frame: Multiverse);

    fn shutdown(&mut self);
}

pub struct OutputInstance {
    implementation: Box<dyn OutputInstanceImplementation>,
}

impl OutputInstance {
    pub fn new<I: OutputInstanceImplementation + 'static>(implementation: I) -> Self {
        Self { implementation: Box::new(implementation) }
    }

    pub fn run(&mut self, output_rx: crossbeam_channel::Receiver<Multiverse>) {
        self.implementation.setup();

        while let Ok(frame) = output_rx.recv() {
            self.implementation.handle_frame(frame);
        }

        self.implementation.shutdown();
    }
}

impl From<OutputInstanceDefinition> for OutputInstance {
    fn from(definition: OutputInstanceDefinition) -> Self {
        match definition {
            OutputInstanceDefinition::EnttecOpenDmx { universe_id, serial_number } => {
                Self::new(enttec_open_dmx::EnttecOpenDmxOutput::new(universe_id, &serial_number))
            }
        }
    }
}
