use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Protocols {
    pub(crate) sacn: Sacn,
}

impl Protocols {
    pub fn sacn(&self) -> &Sacn {
        &self.sacn
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sacn {
    pub(crate) outputs: Vec<SacnOutput>,
}

impl Sacn {
    pub fn outputs(&self) -> &[SacnOutput] {
        &self.outputs
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SacnOutput {
    pub(crate) name: String,
    pub(crate) mode: SacnMode,
    pub(crate) local_universes: Vec<u16>,
    pub(crate) destination_universe: u16,
    pub(crate) priority: u8,
}

impl SacnOutput {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mode(&self) -> SacnMode {
        self.mode
    }

    pub fn local_universes(&self) -> &[u16] {
        &self.local_universes
    }

    pub fn destination_universe(&self) -> u16 {
        self.destination_universe
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SacnMode {
    Unicast { destination_ip: IpAddr },
    Multicast,
}
