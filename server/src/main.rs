mod auth;
mod query;
mod result;
mod routes;
mod state;

use axum::{
    Router,
    routing::{get, post},
};
use clusterizer_common::types::{
    Assignment, Platform, Project, ProjectVersion, Result, Task, User,
};
use routes::*;
use sqlx::PgPool;
use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let database_url = dotenvy::var("DATABASE_URL").unwrap();
    let secret = dotenvy::var("CLUSTERIZER_SECRET").unwrap();
    let address = dotenvy::var("CLUSTERIZER_ADDRESS").unwrap();

    let state = AppState {
        pool: PgPool::connect(&database_url).await.unwrap(),
        secret: secret.into_bytes(),
    };

    let app = Router::new()
        .route("/users", get(get_all::<User>))
        .route("/users/{id}", get(get_one::<User>))
        .route(
            "/users/{id}/assignments",
            get(get_all_by::<Assignment, User>),
        )
        .route("/projects", get(get_all::<Project>))
        .route("/projects/{id}", get(get_one::<Project>))
        .route(
            "/projects/{id}/project_versions",
            get(get_all_by::<ProjectVersion, Project>),
        )
        .route("/projects/{id}/tasks", get(get_all_by::<Task, Project>))
        .route("/platforms", get(get_all::<Platform>))
        .route("/platforms/{id}", get(get_one::<Platform>))
        .route(
            "/platforms/{id}/project_versions",
            get(get_all_by::<ProjectVersion, Platform>),
        )
        .route("/project_versions", get(get_all::<ProjectVersion>))
        .route("/project_versions/{id}", get(get_one::<ProjectVersion>))
        .route("/tasks", get(get_all::<Task>))
        .route("/tasks/{id}", get(get_one::<Task>))
        .route(
            "/tasks/{id}/assignments",
            get(get_all_by::<Assignment, Task>),
        )
        .route("/assignments", get(get_all::<Assignment>))
        .route("/assignments/{id}", get(get_one::<Assignment>))
        .route(
            "/assignments/{id}/result",
            get(get_one_by::<Result, Assignment>),
        )
        .route("/results", get(get_all::<Result>))
        .route("/results/{id}", get(get_one::<Result>))
        //
        .route("/users/profile", get(users::get_profile))
        .route("/users/register", post(users::register))
        .route("/tasks/fetch/{platform_id}", post(tasks::fetch))
        .route("/tasks/{task_id}/submit", post(tasks::submit))
        .with_state(state);

    let listener = TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
