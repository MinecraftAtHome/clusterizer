mod get;

use clusterizer_common::{
    id::Id,
    messages::{RegisterRequest, RegisterResponse, SubmitRequest},
    types::{Platform, Task},
};
use get::{GetAll, GetAllBy, GetOne, GetOneBy};
use reqwest::{IntoUrl, Method, RequestBuilder, Result, header};
use serde::{Serialize, de::DeserializeOwned};

pub struct Client {
    client: reqwest::Client,
    url: String,
    api_key: Option<String>,
}

impl Client {
    pub fn new(url: String, api_key: Option<String>) -> Client {
        Client {
            client: reqwest::Client::new(),
            url,
            api_key,
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key)
    }

    pub async fn get_all<T: GetAll + DeserializeOwned>(&self) -> Result<Vec<T>> {
        self.get(T::get_all(&self.url)).await
    }

    pub async fn get_all_by<T: GetAllBy<U> + DeserializeOwned, U>(
        &self,
        id: Id<U>,
    ) -> Result<Vec<T>> {
        self.get(T::get_all_by(&self.url, id)).await
    }

    pub async fn get_one<T: GetOne + DeserializeOwned>(&self, id: Id<T>) -> Result<T> {
        self.get(T::get_one(&self.url, id)).await
    }

    pub async fn get_one_by<T: GetOneBy<U> + DeserializeOwned, U>(
        &self,
        id: Id<U>,
    ) -> Result<Option<T>> {
        self.get(T::get_one_by(&self.url, id)).await
    }

    pub async fn submit_task(
        &self,
        task_id: Id<Task>,
        submit_request: &SubmitRequest,
    ) -> Result<()> {
        let url = format!("{}/tasks/{task_id}/submit", self.url);
        self.post_data(url, submit_request).await
    }

    pub async fn fetch_tasks(&self, platform_id: Id<Platform>) -> Result<Vec<Task>> {
        let url = format!("{}/tasks/fetch/{platform_id}", self.url);
        self.post(url).await
    }

    pub async fn register_user(
        &self,
        register_request: &RegisterRequest,
    ) -> Result<RegisterResponse> {
        let url = format!("{}/users/register", self.url);
        self.post_data(url, register_request).await
    }

    fn request(&self, method: Method, url: impl IntoUrl) -> RequestBuilder {
        let mut request = self.client.request(method, url);

        if let Some(ref api_key) = self.api_key {
            request = request.bearer_auth(api_key);
        }

        request
    }

    async fn get<Response: DeserializeOwned>(&self, url: impl IntoUrl) -> Result<Response> {
        self.request(Method::GET, url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    async fn post<Response: DeserializeOwned>(&self, url: impl IntoUrl) -> Result<Response> {
        self.request(Method::POST, url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    async fn post_data<Request: Serialize + ?Sized, Response: DeserializeOwned>(
        &self,
        url: impl IntoUrl,
        data: &Request,
    ) -> Result<Response> {
        self.request(Method::POST, url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(data)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}
