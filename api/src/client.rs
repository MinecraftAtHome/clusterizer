use clusterizer_common::{
    errors::{FetchTasksError, Infallible, NotFound, RegisterError, SubmitResultError},
    id::Id,
    requests::{FetchTasksRequest, RegisterRequest, SubmitResultRequest},
    responses::RegisterResponse,
    types::Task,
};
use reqwest::{IntoUrl, RequestBuilder, Response, header};
use serde::{Serialize, de::DeserializeOwned};

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
        Ok(self.get(url).await?.json().await?)
    }

    pub async fn get_all_by<T: GetAllBy<U> + DeserializeOwned, U>(
        &self,
        id: Id<U>,
    ) -> ApiResult<Vec<T>, NotFound> {
        let url = T::get_all_by(&self.url, id);
        Ok(self.get(url).await?.json().await?)
    }

    pub async fn get_one<T: GetOne + DeserializeOwned>(&self, id: Id<T>) -> ApiResult<T, NotFound> {
        let url = T::get_one(&self.url, id);
        Ok(self.get(url).await?.json().await?)
    }

    pub async fn get_one_by<T: GetOneBy<U> + DeserializeOwned, U>(
        &self,
        id: Id<U>,
    ) -> ApiResult<Option<T>, NotFound> {
        let url = T::get_one_by(&self.url, id);
        Ok(self.get(url).await?.json().await?)
    }

    pub async fn register(
        &self,
        request: &RegisterRequest,
    ) -> ApiResult<RegisterResponse, RegisterError> {
        let url = format!("{}/register", self.url);
        Ok(self.post(url, request).await?.json().await?)
    }

    pub async fn fetch_tasks(
        &self,
        request: &FetchTasksRequest,
    ) -> ApiResult<Vec<Task>, FetchTasksError> {
        let url = format!("{}/fetch_tasks", self.url);
        Ok(self.post(url, request).await?.json().await?)
    }

    pub async fn submit_result(
        &self,
        task_id: Id<Task>,
        request: &SubmitResultRequest,
    ) -> ApiResult<(), SubmitResultError> {
        let url = format!("{}/submit_result/{task_id}", self.url);
        self.post(url, request).await?;
        Ok(())
    }

    async fn get<Error: DeserializeOwned>(&self, url: impl IntoUrl) -> ApiResult<Response, Error> {
        self.send(self.client.get(url)).await
    }

    async fn post<Error: DeserializeOwned>(
        &self,
        url: impl IntoUrl,
        request: &impl Serialize,
    ) -> ApiResult<Response, Error> {
        self.send(self.client.post(url).json(request)).await
    }

    async fn send<Error: DeserializeOwned>(
        &self,
        mut request: RequestBuilder,
    ) -> ApiResult<Response, Error> {
        if let Some(ref api_key) = self.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request.send().await?;

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
}
