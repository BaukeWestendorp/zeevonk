use std::collections::HashMap;
use std::sync::{Arc, mpsc};

use crate::ident::Identifier;
use crate::project::Project;
use crate::project::definition::router::Route;
use crate::trigger::Trigger;

pub struct Router {
    project: Arc<Project>,

    registered_processors: HashMap<Identifier, mpsc::Sender<Trigger>>,
}

impl Router {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project, registered_processors: HashMap::new() }
    }

    pub fn handle_trigger(&self, client_id: &Identifier, trigger: Trigger) {
        let Some(route) = self.route_for_trigger_source(client_id, trigger.id()) else {
            log::warn!("unrouted trigger from client '{client_id}': {trigger:?}");
            return;
        };

        for target_client in &route.to_clients {
            let Some(client) = self.registered_processor(target_client) else {
                log::warn!("route to unregistered processor client: {target_client}");
                continue;
            };

            client
                .send(trigger.clone())
                .expect("client should be unregistered before channel is closed");
        }
    }

    fn route_for_trigger_source(
        &self,
        client_id: &Identifier,
        trigger_id: &Identifier,
    ) -> Option<&Route> {
        let routes = &self.project.router_definition().routes;

        routes.iter().find(|route| {
            &route.from_client == client_id && route.from_trigger.as_ref() == Some(trigger_id)
        })
    }

    fn registered_processor(&self, client: &Identifier) -> Option<&mpsc::Sender<Trigger>> {
        self.registered_processors.get(client)
    }
}
