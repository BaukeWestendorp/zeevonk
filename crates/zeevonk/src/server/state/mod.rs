use theymx::Multiverse;
use tokio::sync::RwLock;

use crate::attr::Attribute;
use crate::server;
use crate::server::showfile::Showfile;
use crate::server::state::trigger_router::TriggerRouter;
use crate::show::ShowData;
use crate::show::fixture::FixturePath;
use crate::value::{AttributeValues, ClampedValue};

mod show_data_builder;
mod trigger_router;

pub struct State {
    pub show_data: RwLock<ShowData>,
    pub pending_attribute_values: RwLock<AttributeValues>,
    pub output_multiverse: RwLock<Multiverse>,
    pub trigger_router: RwLock<TriggerRouter>,
}

impl State {
    pub fn new(showfile: &Showfile) -> Result<Self, server::Error> {
        Ok(Self {
            show_data: RwLock::new(ShowData::from_showfile(showfile)?),
            pending_attribute_values: RwLock::new(AttributeValues::new()),
            output_multiverse: RwLock::new(Multiverse::new()),
            trigger_router: RwLock::new(TriggerRouter::new()),
        })
    }

    pub async fn set_attribute_value(
        &self,
        fixture_path: FixturePath,
        attribute: Attribute,
        value: ClampedValue,
    ) {
        self.pending_attribute_values.write().await.set(fixture_path, attribute, value);
    }
}
