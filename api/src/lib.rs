use clusterizer_common::{
    messages::{RegisterRequest, RegisterResponse, SubmitRequest},
    types::{
        Assignment, Platform, Project, ProjectVersion, Result as ClusterizerResult, Task, User,
    },
};
use reqwest::{Error, Method, RequestBuilder, header};
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

    // GET requests
    pub async fn get_users(&self) -> Result<Vec<User>, Error> {
        let uri = "/users";
        self.get(uri).await
    }

    pub async fn get_user(&self, user_id: i64) -> Result<User, Error> {
        let uri = format!("/users/{user_id}");
        self.get(&uri).await
    }

    pub async fn get_user_profile(&self) -> Result<User, Error> {
        let uri = "/users/profile";
        self.get(uri).await
    }

    pub async fn get_projects(&self) -> Result<Vec<Project>, Error> {
        let uri = "/projects";
        self.get(uri).await
    }

    pub async fn get_project(&self, project_id: i64) -> Result<Project, Error> {
        let uri = format!("/projects/{project_id}");
        self.get(&uri).await
    }

    pub async fn get_project_results(
        &self,
        project_id: i64,
    ) -> Result<Vec<ClusterizerResult>, Error> {
        let uri = format!("/projects/{project_id}/results");
        self.get(&uri).await
    }

    pub async fn get_project_project_versions(
        &self,
        project_id: i64,
    ) -> Result<Vec<ProjectVersion>, Error> {
        let uri = format!("/projects/{project_id}/project_versions");
        self.get(&uri).await
    }

    pub async fn get_platforms(&self) -> Result<Vec<Platform>, Error> {
        let uri = "/platforms";
        self.get(uri).await
    }

    pub async fn get_platform(&self, platform_id: i64) -> Result<Platform, Error> {
        let uri = format!("/platforms/{platform_id}");
        self.get(&uri).await
    }

    pub async fn get_project_versions(&self) -> Result<Vec<ProjectVersion>, Error> {
        let uri = "/project_versions";
        self.get(uri).await
    }

    pub async fn get_project_version(
        &self,
        project_version_id: i64,
    ) -> Result<ProjectVersion, Error> {
        let uri = format!("/project_versions/{project_version_id}");
        self.get(&uri).await
    }

    pub async fn get_tasks(&self) -> Result<Vec<Task>, Error> {
        let uri = "/tasks";
        self.get(uri).await
    }

    pub async fn get_task(&self, task_id: i64) -> Result<Task, Error> {
        let uri = format!("/tasks/{task_id}");
        self.get(&uri).await
    }

    pub async fn get_assignments(&self) -> Result<Vec<Assignment>, Error> {
        let uri = "/assignments";
        self.get(uri).await
    }

    pub async fn get_assignment(&self, assignment_id: i64) -> Result<Assignment, Error> {
        let uri = format!("/assignments/{assignment_id}");
        self.get(&uri).await
    }

    pub async fn get_results(&self) -> Result<Vec<ClusterizerResult>, Error> {
        let uri = "/results";
        self.get(uri).await
    }

    pub async fn get_result(&self, result_id: i64) -> Result<ClusterizerResult, Error> {
        let uri = format!("/results/{result_id}");
        self.get(&uri).await
    }

    // POST requests
    pub async fn submit_task(
        &self,
        task_id: i64,
        submit_request: &SubmitRequest,
    ) -> Result<(), Error> {
        let uri = format!("/tasks/{task_id}/submit");
        self.post_data(&uri, submit_request).await
    }

    pub async fn fetch_tasks(&self) -> Result<Vec<Task>, Error> {
        let uri = "/tasks/fetch";
        self.post(uri).await
    }

    pub async fn register_user(
        &self,
        register_request: &RegisterRequest,
    ) -> Result<RegisterResponse, Error> {
        let uri = "/users/register";
        self.post_data(uri, register_request).await
    }

    fn request(&self, method: Method, uri: &str) -> RequestBuilder {
        let mut request = self.client.request(method, format!("{}{}", self.url, uri));

        if let Some(ref api_key) = self.api_key {
            request = request.bearer_auth(api_key);
        }

        request
    }

    async fn get<Response: DeserializeOwned>(&self, uri: &str) -> Result<Response, Error> {
        self.request(Method::GET, uri).send().await?.json().await
    }

    async fn post<Response: DeserializeOwned>(&self, uri: &str) -> Result<Response, Error> {
        self.request(Method::POST, uri).send().await?.json().await
    }

    async fn post_data<Request: Serialize + ?Sized, Response: DeserializeOwned>(
        &self,
        uri: &str,
        data: &Request,
    ) -> Result<Response, Error> {
        self.request(Method::POST, uri)
            .header(header::CONTENT_TYPE, "application/json")
            .json(data)
            .send()
            .await?
            .json()
            .await
    }
}
