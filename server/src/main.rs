mod auth;
mod result;
mod routes;
mod state;
mod util;

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
use tokio::net::TcpListener;
use tokio::time;
#[tokio::main]
async fn main() {
    let database_url = dotenvy::var("DATABASE_URL").unwrap();
    let secret = dotenvy::var("CLUSTERIZER_SECRET").unwrap();
    let address = dotenvy::var("CLUSTERIZER_ADDRESS").unwrap();
    let state = AppState {
        pool: PgPool::connect(&database_url).await.unwrap(),
        secret: secret.into_bytes(),
    };

    let deadline_task_state = state.clone();
    let mut deadline_interval = time::interval(time::Duration::from_secs(60 * 15));
    tokio::spawn(async move {
        loop {
            deadline_interval.tick().await;
            let _result = util::update_assignments_exceed_deadline(&deadline_task_state).await;
            //TODO log result
        }
    });

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

    axum::serve(listener, app).await.unwrap();
}
