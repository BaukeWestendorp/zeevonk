use std::net::IpAddr;

use crate::showfile::Label;

#[derive(Debug, Clone, PartialEq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Protocols {
    sacn: Sacn,
}

impl Protocols {
    pub fn sacn(&self) -> &Sacn {
        &self.sacn
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Sacn {
    outputs: Vec<SacnOutput>,
}

impl Sacn {
    pub fn outputs(&self) -> &[SacnOutput] {
        &self.outputs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SacnOutput {
    label: Label,
    mode: SacnMode,
    local_universes: Vec<u16>,
    destination_universe: u16,
    priority: u8,
}

impl SacnOutput {
    pub fn label(&self) -> &Label {
        &self.label
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
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SacnMode {
    Unicast { destination_ip: IpAddr },
    Multicast,
}
