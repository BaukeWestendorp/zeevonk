/// General configuration for the server.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    controller_port: u16,
}

impl Config {
    /// Returns the port for controller connections.
    pub fn controller_port(&self) -> u16 {
        self.controller_port
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { controller_port: crate::DEFAULT_CONTROLLER_PORT }
    }
}
