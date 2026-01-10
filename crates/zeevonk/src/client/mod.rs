use std::net::SocketAddr;

use crate::dmx::Multiverse;
use crate::show::ShowData;
use crate::value::AttributeValues;

pub struct Client {
    address: SocketAddr,
    client: reqwest::Client,
}

impl Client {
    pub fn new(address: SocketAddr) -> Self {
        Self { address, client: reqwest::Client::new() }
    }

    pub async fn show_data(&self) -> Result<ShowData, crate::Error> {
        self.get_json("/show-data").await
    }

    pub async fn dmx_output(&self) -> Result<Multiverse, crate::Error> {
        self.get_json("/dmx-output").await
    }

    pub async fn set_attribute_values(&self, values: &AttributeValues) -> Result<(), crate::Error> {
        self.client
            .post(self.url("/attribute-values"))
            .json(values)
            .send()
            .await
            .map_err(|err| crate::Error::client(err.to_string()))?;
        Ok(())
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.address, path)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, crate::Error> {
        self.client
            .get(self.url(path))
            .send()
            .await
            .map_err(|err| crate::Error::client(err.to_string()))?
            .json()
            .await
            .map_err(|err| crate::Error::client(err.to_string()))
    }
}
