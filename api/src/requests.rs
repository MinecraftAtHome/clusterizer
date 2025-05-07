use clusterizer_common::messages::{SubmitRequest, SubmitResponse};
use clusterizer_common::types::{Assignment, Project, ProjectVersion, Task};
use reqwest::Error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

struct Client {
    client: reqwest::Client,
    url: String,
    api_key: Option<String>,
}

impl Client {
    pub async fn get<T: DeserializeOwned>(&self, uri: &str) -> Result<T, Error> {
        let full = format!("{}{}", self.url, uri);
        let client = reqwest::Client::new();
        let mut req = client.get(full);
        if let Some(ref x) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", x));
        }
        let res = req.send().await?.json::<T>().await?;
        Ok(res)
    }

    pub async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        uri: &str,
        data: &T,
    ) -> Result<R, Error> {
        let full = format!("{}{}", self.url, uri);
        let client = reqwest::Client::new();
        let mut req = client
            .post(full)
            .json(data)
            .header("Content-Type", "application/json");
        if let Some(ref x) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", x));
        }
        let res = req.send().await?.json::<R>().await?;
        Ok(res)
    }

    //GET requests
    pub async fn get_task(&self, task_id: i64) -> Result<Task, Error> {
        let uri = format!("/tasks/{}", task_id);
        Ok(self.get(&uri).await?)
    }

    pub async fn get_project(&self, project_id: i64) -> Result<Project, Error> {
        let uri = format!("/projects/{}", project_id);
        Ok(self.get(&uri).await?)
    }

    pub async fn get_project_version(&self, project_id: i64) -> Result<Vec<ProjectVersion>, Error> {
        let uri = format!("/projects/{}/project_versions", project_id);
        Ok(self.get(&uri).await?)
    }

    pub async fn fetch_assignments(&self) -> Result<Vec<Assignment>, Error> {
        let uri = "/assignments/fetch";
        Ok(self.get(&uri).await?)
    }

    //POST requests
    pub async fn submit_result(
        &self,
        task_id: i64,
        result_data: &SubmitRequest,
    ) -> Result<SubmitResponse, Error> {
        let uri = "/results/submit";
        Ok(self.post(&uri, result_data).await?)
    }
}
