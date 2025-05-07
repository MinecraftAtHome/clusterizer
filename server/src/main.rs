mod auth;
mod result;
mod routes;
mod state;

use std::{env, sync::Arc};

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use state::AppState;
use tokio::{net::TcpListener, sync::Mutex};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let database_url = env::var("DATABASE_URL").unwrap();
    let secret = env::var("CLUSTERIZER_SECRET").unwrap();

    let state = AppState {
        pool: PgPool::connect(&database_url).await.unwrap(),
        secret: secret.into_bytes(),
        fetch_mutex: Arc::new(Mutex::new(())),
    };

    let app = Router::new()
        .route("/users", get(routes::users::get_all))
        .route("/users/{id}", get(routes::users::get_one))
        .route("/users/profile", get(routes::users::profile))
        .route("/users/register", post(routes::users::register))
        .route("/projects", get(routes::projects::get_all))
        .route("/projects/{id}", get(routes::projects::get_one))
        .route("/projects/{id}/results", get(routes::projects::results))
        .route("/platforms", get(routes::platforms::get_all))
        .route("/platforms/{id}", get(routes::platforms::get_one))
        .route("/project_versions", get(routes::project_versions::get_all))
        .route(
            "/project_versions/{id}",
            get(routes::project_versions::get_one),
        )
        .route("/tasks", get(routes::tasks::get_all))
        .route("/tasks/{id}", get(routes::tasks::get_one))
        .route("/assignments", get(routes::assignments::get_all))
        .route("/assignments/{id}", get(routes::assignments::get_one))
        .route("/assignments/fetch", post(routes::assignments::fetch))
        .route("/results", get(routes::results::get_all))
        .route("/results/{id}", get(routes::results::get_one))
        .route("/results/submit", post(routes::results::submit))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
