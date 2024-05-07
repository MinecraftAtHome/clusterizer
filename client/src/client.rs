use crate::error::{Error, Result};
use clusterizer_common::types::{CreateUser, CreateUserResponse};
use reqwest::{IntoUrl, Url};
use serde::{de::DeserializeOwned, Serialize};

pub struct Client {
    client: reqwest::Client,
    url: Url,
}

impl Client {
    pub fn new<U: IntoUrl>(url: U) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            url: url.into_url()?,
        })
    }

    pub async fn create_user(&self, create_user: &CreateUser) -> Result<CreateUserResponse> {
        self.post("users", create_user).await
    }

    async fn post<T, U>(&self, route: &str, body: &T) -> Result<U>
    where
        T: Serialize + ?Sized,
        U: DeserializeOwned,
    {
        let url = self.url.join(route).unwrap();
        let request = self.client.post(url).json(body);
        let response = request.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Error::ClusterizerError(response.json().await?))
        }
    }
}
