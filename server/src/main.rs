mod auth;
mod result;
mod routes;
mod state;

use std::{env, sync::Arc};

use axum::{
    Router,
    routing::{get, post},
};
use routes::*;
use sqlx::PgPool;
use state::AppState;
use tokio::{net::TcpListener, sync::Mutex};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let database_url = env::var("DATABASE_URL").unwrap();
    let secret = env::var("CLUSTERIZER_SECRET").unwrap();
    let address = env::var("CLUSTERIZER_ADDRESS").unwrap();

    let state = AppState {
        pool: PgPool::connect(&database_url).await.unwrap(),
        secret: secret.into_bytes(),
        fetch_mutex: Arc::new(Mutex::new(())),
    };

    let app = Router::new()
        .route("/users", get(users::get_all))
        .route("/users/{id}", get(users::get_one))
        .route("/users/profile", get(users::profile))
        .route("/users/register", post(users::register))
        .route("/projects", get(projects::get_all))
        .route("/projects/{id}", get(projects::get_one))
        .route("/projects/{id}/results", get(projects::results))
        .route("/platforms", get(platforms::get_all))
        .route("/platforms/{id}", get(platforms::get_one))
        .route("/project_versions", get(project_versions::get_all))
        .route("/project_versions/{id}", get(project_versions::get_one))
        .route(
            "/projects/{id}/project_versions",
            get(routes::projects::versions),
        )
        .route("/tasks", get(tasks::get_all))
        .route("/tasks/{id}", get(tasks::get_one))
        .route("/tasks/fetch", post(tasks::fetch))
        .route("/tasks/{id}/submit", post(tasks::submit))
        .route("/assignments", get(assignments::get_all))
        .route("/assignments/{id}", get(assignments::get_one))
        .route("/results", get(results::get_all))
        .route("/results/{id}", get(results::get_one))
        .with_state(state);

    let listener = TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
