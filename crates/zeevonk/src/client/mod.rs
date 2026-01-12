use std::net::SocketAddr;

use theymx::Multiverse;

use crate::show::ShowData;
use crate::trigger::Trigger;
use crate::value::AttributeValues;
use crate::{Identifier, client};

mod error;

pub use error::Error;

pub struct Client {
    id: Identifier,

    address: SocketAddr,
    client: reqwest::Client,
}

impl Client {
    pub fn new(id: Identifier, address: SocketAddr) -> Self {
        Self { id: id.into(), address, client: reqwest::Client::new() }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub async fn show_data(&self) -> Result<ShowData, client::Error> {
        self.get_json("/show-data").await
    }

    pub async fn dmx_output(&self) -> Result<Multiverse, client::Error> {
        self.get_json("/dmx-output").await
    }

    pub async fn set_attribute_values(
        &self,
        values: &AttributeValues,
    ) -> Result<(), client::Error> {
        self.client
            .post(self.url("/attribute-values"))
            .json(values)
            .send()
            .await
            .map_err(|err| client::Error::RequestFailed(err.to_string()))?;

        Ok(())
    }

    pub async fn send_trigger(&self, trigger: Trigger) -> Result<(), client::Error> {
        self.client
            .post(self.url(&format!("/trigger/{}:{}", self.id, trigger)))
            .send()
            .await
            .map_err(|err| client::Error::RequestFailed(err.to_string()))?;

        Ok(())
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.address, path)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, client::Error> {
        let response = self
            .client
            .get(self.url(path))
            .send()
            .await
            .map_err(|err| client::Error::RequestFailed(err.to_string()))?;

        response
            .json::<T>()
            .await
            .map_err(|err| client::Error::DeserializationFailed(err.to_string()))
    }
}
