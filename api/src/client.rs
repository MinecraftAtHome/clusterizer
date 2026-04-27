use clusterizer_common::{
    errors::{
        CreateFileError, FetchTasksError, RegisterError, SubmitResultError, ValidateFetchError,
        ValidateSubmitError,
    },
    records::{File, Get, Project, Task},
    requests::{
        CreateFileRequest, FetchTasksRequest, RegisterRequest, SubmitResultRequest,
        ValidateSubmitRequest,
    },
    responses::RegisterResponse,
    types::Id,
};
use reqwest::{IntoUrl, RequestBuilder, Response, Url, header};
use serde::{Serialize, de::DeserializeOwned};

use crate::result::{ApiError, ApiResult};

pub struct ApiClient {
    client: reqwest::Client,
    url: Url,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(url: Url, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            api_key,
        }
    }

    pub async fn get<T: Get>(&self, by: &T) -> ApiResult<T::Ok, T::Err> {
        let request = by.get(&self.client, &self.url)?;
        Ok(self.send(request).await?.json().await?)
    }

    pub async fn register(
        &self,
        request: &RegisterRequest,
    ) -> ApiResult<RegisterResponse, RegisterError> {
        let url = self.url.join("register")?;
        Ok(self.send_post(url, request).await?.json().await?)
    }

    pub async fn fetch_tasks(
        &self,
        request: &FetchTasksRequest,
    ) -> ApiResult<Vec<Task>, FetchTasksError> {
        let url = self.url.join("fetch_tasks")?;
        Ok(self.send_post(url, request).await?.json().await?)
    }

    pub async fn submit_result(
        &self,
        task_id: Id<Task>,
        request: &SubmitResultRequest,
    ) -> ApiResult<(), SubmitResultError> {
        let url = self.url.join("submit_result")?.join(&task_id.to_string())?;
        self.send_post(url, request).await?;
        Ok(())
    }

    pub async fn validate_fetch(
        &self,
        project_id: Id<Project>,
    ) -> ApiResult<Vec<Task>, ValidateFetchError> {
        let url = self
            .url
            .join("validate_fetch")?
            .join(&project_id.to_string())?;
        Ok(self.send_get(url).await?.json().await?)
    }

    pub async fn validate_submit(
        &self,
        request: &ValidateSubmitRequest,
    ) -> ApiResult<(), ValidateSubmitError> {
        let url = self.url.join("validate_submit")?;
        self.send_post(url, request).await?;
        Ok(())
    }

    pub async fn create_file(
        &self,
        request: &CreateFileRequest,
    ) -> ApiResult<Id<File>, CreateFileError> {
        let url = self.url.join("files")?;
        Ok(self.send_post(url, request).await?.json().await?)
    }

    async fn send_get<Error: DeserializeOwned>(
        &self,
        url: impl IntoUrl,
    ) -> ApiResult<Response, Error> {
        self.send(self.client.get(url)).await
    }

    async fn send_post<Error: DeserializeOwned>(
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
