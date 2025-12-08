use crate::showfile::Showfile;

pub struct Engine {}

impl Engine {
    pub fn new(_showfile: Showfile) -> Self {
        Self {}
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        loop {}
    }
}
