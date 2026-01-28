use std::sync::Arc;

use crate::ident::Identifier;
use crate::project::Project;
use crate::project::file::router::Route;
use crate::server::client::ClientAgent;
use crate::trigger::Trigger;

pub struct Router {
    project: Arc<Project>,
    client_agent: Arc<ClientAgent>,
}

impl Router {
    pub fn new(project: Arc<Project>, client_agent: Arc<ClientAgent>) -> Self {
        Self { project, client_agent }
    }

    pub async fn handle_trigger(&self, client_id: &Identifier, trigger: Trigger) {
        let Some(route) = self.route_for_trigger_source(client_id, trigger.id()) else {
            log::warn!("unrouted trigger from client '{client_id}': {trigger:?}");
            return;
        };

        for to_client_id in &route.to_clients {
            if let Err(e) = self
                .client_agent
                .send_trigger(route.from_client.clone(), to_client_id.clone(), trigger.clone())
                .await
            {
                log::debug!("route to unregistered client: '{to_client_id}': {e}");
            }
        }
    }

    fn route_for_trigger_source(
        &self,
        client_id: &Identifier,
        trigger_id: &Identifier,
    ) -> Option<&Route> {
        let routes = &self.project.file().router.routes;

        routes.iter().find(|route| {
            let trigger_matches =
                route.from_trigger.as_ref() == Some(trigger_id) || route.from_trigger.is_none();
            &route.from_client == client_id && trigger_matches
        })
    }
}
