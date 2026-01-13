use std::collections::HashMap;
use std::net::SocketAddr;

use crate::trigger::Trigger;

/// Routes triggers from controller clients and processor clients to the appropriate handler.
pub struct TriggerRouter {
    clients: HashMap<SocketAddr, ClientInfo>,
}

#[derive(Debug)]
struct ClientInfo {
    name: String,
    role: ClientRole,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ClientRole {
    Controller,
    Processor,
}

impl ClientRole {
    fn as_str(&self) -> &'static str {
        match self {
            ClientRole::Controller => "controller",
            ClientRole::Processor => "processor",
        }
    }
}

impl TriggerRouter {
    pub fn new() -> Self {
        Self { clients: HashMap::new() }
    }

    fn register_client(&mut self, address: SocketAddr, name: String, role: ClientRole) {
        match self.clients.insert(address, ClientInfo { name: name.clone(), role }) {
            Some(old) => {
                log::warn!(
                    "{} client re-registered: {} -> {} ({})",
                    role.as_str(),
                    old.name,
                    name,
                    address
                );
            }
            None => {
                log::info!("registered {} client {} ({})", role.as_str(), name, address);
            }
        }
    }

    fn unregister_client(&mut self, address: SocketAddr, role: ClientRole) {
        match self.clients.remove(&address) {
            Some(info) => {
                log::info!("unregistered {} client {} ({})", role.as_str(), info.name, address);
            }
            None => {
                log::warn!(
                    "attempted to unregister unknown {} client ({})",
                    role.as_str(),
                    address
                );
            }
        }
    }

    pub fn register_controller_client(&mut self, address: SocketAddr, name: String) {
        self.register_client(address, name, ClientRole::Controller);
    }

    pub fn unregister_controller_client(&mut self, address: SocketAddr) {
        self.unregister_client(address, ClientRole::Controller);
    }

    pub fn register_processor_client(&mut self, address: SocketAddr, name: String) {
        self.register_client(address, name, ClientRole::Processor);
    }

    pub fn unregister_processor_client(&mut self, address: SocketAddr) {
        self.unregister_client(address, ClientRole::Processor);
    }

    pub fn handle_trigger(&self, address: SocketAddr, trigger: Trigger) {
        if let Some(info) = self.clients.get(&address) {
            log::info!(
                "received trigger {:?} from {} client {} ({})",
                trigger,
                info.role.as_str(),
                info.name,
                address
            );
        } else {
            log::warn!("received trigger {:?} from unknown client ({})", trigger, address);
        }
    }
}
