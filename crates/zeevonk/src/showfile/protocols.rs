use std::net::IpAddr;

/// Contains all DMX IO protocol configurations.
#[derive(Debug, Clone, PartialEq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Protocols {
    sacn: Sacn,
}

impl Protocols {
    /// Returns a reference to the sACN protocol configuration.
    pub fn sacn(&self) -> &Sacn {
        &self.sacn
    }
}

/// Inputs and outputs for the sACN protocol.
#[derive(Debug, Clone, PartialEq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Sacn {
    outputs: Vec<SacnOutput>,
}

impl Sacn {
    /// Returns all sACN output configurations.
    pub fn outputs(&self) -> &[SacnOutput] {
        &self.outputs
    }
}

/// Configuration for a single sACN output.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SacnOutput {
    label: String,
    mode: SacnMode,
    local_universe: u16,
    destination_universe: u16,
    priority: u8,
    preview_data: bool,
}

impl SacnOutput {
    /// Returns the label for this output.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the sACN mode for this output.
    pub fn mode(&self) -> SacnMode {
        self.mode
    }

    /// Returns the local universes for this output.
    ///
    /// These are Zeevonk's universes that will be sent to the target endpoint.
    pub fn local_universe(&self) -> u16 {
        self.local_universe
    }

    /// Returns the destination universe for this output.
    ///
    /// This is the destination universe for the target endpoint.
    pub fn destination_universe(&self) -> u16 {
        self.destination_universe
    }

    /// Returns the sACN priority for this output.
    pub fn priority(&self) -> u8 {
        self.priority
    }

    /// Returns whether this sACN output is meant
    /// for preview use cases (like visualizers).
    pub fn preview_data(&self) -> bool {
        self.preview_data
    }
}

/// Mode for sACN output.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SacnMode {
    /// Unicast mode with a specific destination IP address.
    Unicast {
        /// The ip address of the targeted sACN endpoint.
        destination_ip: IpAddr,
    },
    /// Multicast mode.
    Multicast,
}
