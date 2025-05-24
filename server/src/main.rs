mod auth;
mod result;
mod routes;
mod state;
mod util;

use std::time::Duration;

use axum::{
    Router,
    routing::{get, post},
};
use clusterizer_common::records::{
    Assignment, Platform, Project, ProjectVersion, Result, Task, User,
};
use routes::{get_all, get_one};
use sqlx::PgPool;
use state::AppState;
use tokio::{net::TcpListener, time};

#[tokio::main]
async fn main() {
    let database_url = dotenvy::var("DATABASE_URL").unwrap();
    let secret = dotenvy::var("CLUSTERIZER_SECRET").unwrap();
    let address = dotenvy::var("CLUSTERIZER_ADDRESS").unwrap();

    let state = AppState {
        pool: PgPool::connect(&database_url).await.unwrap(),
        secret: secret.into_bytes(),
    };

    tokio::join!(
        serve_task(state.clone(), address),
        update_expired_assignments_task(state.clone()),
    );
}

async fn serve_task(state: AppState, address: String) {
    let app = Router::new()
        .route("/users", get(get_all::<User>))
        .route("/users/{id}", get(get_one::<User>))
        .route("/projects", get(get_all::<Project>))
        .route("/projects/{id}", get(get_one::<Project>))
        .route("/platforms", get(get_all::<Platform>))
        .route("/platforms/{id}", get(get_one::<Platform>))
        .route("/project_versions", get(get_all::<ProjectVersion>))
        .route("/project_versions/{id}", get(get_one::<ProjectVersion>))
        .route("/tasks", get(get_all::<Task>))
        .route("/tasks/{id}", get(get_one::<Task>))
        .route("/assignments", get(get_all::<Assignment>))
        .route("/assignments/{id}", get(get_one::<Assignment>))
        .route("/results", get(get_all::<Result>))
        .route("/results/{id}", get(get_one::<Result>))
        .route("/register", post(routes::register))
        .route("/fetch_tasks", post(routes::fetch_tasks))
        .route("/submit_result/{id}", post(routes::submit_result))
        .with_state(state);

    let listener = TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app).await.unwrap()
}

async fn update_expired_assignments_task(state: AppState) {
    let mut interval = time::interval(Duration::from_secs(60 * 15));

    loop {
        interval.tick().await;
        util::update_expired_assignments(&state).await.unwrap();
    }
}
