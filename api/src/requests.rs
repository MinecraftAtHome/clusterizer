use reqwest::Error;
use clusterizer_common::types::{Assignment, Project, ProjectVersion, Task};
use clusterizer_common::messages::{SubmitRequest, SubmitResponse};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

pub async fn get<T: DeserializeOwned>(full: &str, api_key: &str) -> Result<T, Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(full)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?
        .json::<T>()
        .await?;
    Ok(res)
}

pub async fn post<T: Serialize, R: DeserializeOwned>(full: &str, api_key: &str, data: &T) -> Result<R, Error>{
    let client = reqwest::Client::new();
    let res = client
        .post(full)
        .json(data)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?
        .json::<R>()
        .await?;
    Ok(res)
}

//GET requests
pub async fn get_task(base_url: &str, api_key: &str, task_id: i64) -> Result<Task, Error> {
    
    let full = format!("{}/tasks/{}", base_url.trim_end_matches('/'), task_id);
    Ok(get(&full, api_key).await?)
}

pub async fn get_project(base_url: &str, api_key: &str, project_id: i64) -> Result<Project, Error> {

    let full = format!("{}/projects/{}", base_url.trim_end_matches('/'), project_id);
    Ok(get(&full, api_key).await?)
}

pub async fn get_project_version(base_url: &str, api_key: &str, project_id: i64) -> Result<Vec<ProjectVersion>, Error> {

    let full = format!("{}/projects/{}/project_versions", base_url.trim_end_matches('/'), project_id);
    Ok(get(&full, api_key).await?)
}

pub async fn fetch_assignments(base_url: &str, api_key: &str) -> Result<Vec<Assignment>, Error> {

    let full = format!("{}/assignments/fetch", base_url.trim_end_matches('/'));
    Ok(get(&full, api_key).await?)
}

//POST requests
pub async fn submit_result(task_id: i64, base_url: &str, api_key: &str, result_data: &SubmitRequest) -> Result<SubmitResponse, Error> {
    
    let full = format!("{}/results/submit", base_url.trim_end_matches('/'));
    Ok(post(&full.to_string(), api_key, result_data).await?)
}