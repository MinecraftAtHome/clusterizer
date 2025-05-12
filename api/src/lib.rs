mod get;
pub mod result;

use clusterizer_common::{
    errors::{Infallible, NotFound},
    id::Id,
    messages::{RegisterRequest, RegisterResponse, SubmitRequest},
    types::{Platform, Task},
};
use get::{GetAll, GetAllBy, GetOne, GetOneBy};
use reqwest::{IntoUrl, Method, RequestBuilder, header};
use result::{ApiError, ApiResult};
use serde::{Serialize, de::DeserializeOwned};

pub struct Client {
    client: reqwest::Client,
    url: String,
    api_key: Option<String>,
}

impl Client {
    pub fn new(url: String, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            api_key,
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key)
    }

    pub async fn get_all<T: GetAll + DeserializeOwned>(&self) -> ApiResult<Vec<T>, Infallible> {
        self.get(T::get_all(&self.url)).await
    }

    pub async fn get_all_by<T: GetAllBy<U> + DeserializeOwned, U>(
        &self,
        id: Id<U>,
    ) -> ApiResult<Vec<T>, NotFound> {
        self.get(T::get_all_by(&self.url, id)).await
    }

    pub async fn get_one<T: GetOne + DeserializeOwned>(&self, id: Id<T>) -> ApiResult<T, NotFound> {
        self.get(T::get_one(&self.url, id)).await
    }

    pub async fn get_one_by<T: GetOneBy<U> + DeserializeOwned, U>(
        &self,
        id: Id<U>,
    ) -> ApiResult<Option<T>, NotFound> {
        self.get(T::get_one_by(&self.url, id)).await
    }

    pub async fn submit_task(
        &self,
        task_id: Id<Task>,
        submit_request: &SubmitRequest,
    ) -> ApiResult<(), Infallible> {
        let url = format!("{}/tasks/{task_id}/submit", self.url);
        self.post_data(url, submit_request).await
    }

    pub async fn fetch_tasks(&self, platform_id: Id<Platform>) -> ApiResult<Vec<Task>, Infallible> {
        let url = format!("{}/tasks/fetch/{platform_id}", self.url);
        self.post(url).await
    }

    pub async fn register_user(
        &self,
        register_request: &RegisterRequest,
    ) -> ApiResult<RegisterResponse, Infallible> {
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

    async fn get<Response: DeserializeOwned, Error: DeserializeOwned>(
        &self,
        url: impl IntoUrl,
    ) -> ApiResult<Response, Error> {
        send_request(self.request(Method::GET, url)).await
    }

    async fn post<Response: DeserializeOwned, Error: DeserializeOwned>(
        &self,
        url: impl IntoUrl,
    ) -> ApiResult<Response, Error> {
        send_request(self.request(Method::POST, url)).await
    }

    async fn post_data<
        Request: Serialize + ?Sized,
        Response: DeserializeOwned,
        Error: DeserializeOwned,
    >(
        &self,
        url: impl IntoUrl,
        data: &Request,
    ) -> ApiResult<Response, Error> {
        send_request(
            self.request(Method::POST, url)
                .header(header::CONTENT_TYPE, "application/json")
                .json(data),
        )
        .await
    }
}

async fn send_request<Body: DeserializeOwned, Error: DeserializeOwned>(
    builder: RequestBuilder,
) -> ApiResult<Body, Error> {
    let mut response = builder.send().await?;

    if response
        .headers()
        .get(header::CONTENT_TYPE)
        .is_none_or(|value| value != "application/json")
    {
        response = response.error_for_status()?;
    }

    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(ApiError::Specific(response.json().await?))
    }
}
