use clusterizer_common::{
    errors::{FetchTasksError, Infallible, NotFound, RegisterError, SubmitResultError},
    id::Id,
    requests::{FetchTasksRequest, RegisterRequest, SubmitResultRequest},
    responses::RegisterResponse,
    types::Task,
};
use reqwest::{IntoUrl, Method, RequestBuilder, Response, header};
use serde::de::DeserializeOwned;

use crate::{
    get::{GetAll, GetAllBy, GetOne, GetOneBy},
    result::{ApiError, ApiResult},
};

pub struct ApiClient {
    client: reqwest::Client,
    url: String,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(url: String, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            api_key,
        }
    }

    pub async fn get_all<T: GetAll + DeserializeOwned>(&self) -> ApiResult<Vec<T>, Infallible> {
        let url = T::get_all(&self.url);
        Ok(send(self.get(url)).await?.json().await?)
    }

    pub async fn get_all_by<T: GetAllBy<U> + DeserializeOwned, U>(
        &self,
        id: Id<U>,
    ) -> ApiResult<Vec<T>, NotFound> {
        let url = T::get_all_by(&self.url, id);
        Ok(send(self.get(url)).await?.json().await?)
    }

    pub async fn get_one<T: GetOne + DeserializeOwned>(&self, id: Id<T>) -> ApiResult<T, NotFound> {
        let url = T::get_one(&self.url, id);
        Ok(send(self.get(url)).await?.json().await?)
    }

    pub async fn get_one_by<T: GetOneBy<U> + DeserializeOwned, U>(
        &self,
        id: Id<U>,
    ) -> ApiResult<Option<T>, NotFound> {
        let url = T::get_one_by(&self.url, id);
        Ok(send(self.get(url)).await?.json().await?)
    }

    pub async fn register(
        &self,
        request: &RegisterRequest,
    ) -> ApiResult<RegisterResponse, RegisterError> {
        let url = format!("{}/register", self.url);
        Ok(send(self.post(url).json(request)).await?.json().await?)
    }

    pub async fn fetch_tasks(
        &self,
        request: &FetchTasksRequest,
    ) -> ApiResult<Vec<Task>, FetchTasksError> {
        let url = format!("{}/fetch_tasks", self.url);
        Ok(send(self.post(url).json(request)).await?.json().await?)
    }

    pub async fn submit_result(
        &self,
        task_id: Id<Task>,
        request: &SubmitResultRequest,
    ) -> ApiResult<(), SubmitResultError> {
        let url = format!("{}/submit_result/{task_id}", self.url);
        send(self.post(url).json(request)).await?;
        Ok(())
    }

    fn request(&self, method: Method, url: impl IntoUrl) -> RequestBuilder {
        let mut request = self.client.request(method, url);

        if let Some(ref api_key) = self.api_key {
            request = request.bearer_auth(api_key);
        }

        request
    }

    fn get(&self, url: impl IntoUrl) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    fn post(&self, url: impl IntoUrl) -> RequestBuilder {
        self.request(Method::POST, url)
    }
}

async fn send<Error: DeserializeOwned>(builder: RequestBuilder) -> ApiResult<Response, Error> {
    let response = builder.send().await?;

    if let Some(err) = response.error_for_status_ref().err() {
        if response
            .headers()
            .get(header::CONTENT_TYPE)
            .is_some_and(|value| value == "application/json")
        {
            Err(ApiError::Specific(response.json().await?))
        } else {
            let string = response.text().await?;

            if string.is_empty() {
                Err(ApiError::Reqwest(err))
            } else {
                Err(ApiError::String(string))
            }
        }
    } else {
        Ok(response)
    }
}
