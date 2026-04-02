mod auth;
mod result;
mod routes;
mod state;
mod tasks;

use axum::{
    Router,
    routing::{get, post},
};
use clusterizer_common::{
    records::{
        Assignment, File, Platform, PlatformRunner, Project, ProjectRunner, Record, Result, Select,
        Task, User,
    },
    types::Id,
};

use serde::{Serialize, de::DeserializeOwned};
use sqlx::PgPool;
use state::AppState;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = dotenvy::var("DATABASE_URL").unwrap();
    let secret = dotenvy::var("CLUSTERIZER_SECRET").unwrap();
    let address = dotenvy::var("CLUSTERIZER_ADDRESS").unwrap();

    let state = AppState {
        pool: PgPool::connect(&database_url).await.unwrap(),
        secret: secret.into_bytes(),
    };

    tokio::join!(
        serve_task(state.clone(), address),
        tasks::update_expired_assignments(state.clone()),
    );
}

async fn serve_task(state: AppState, address: String) {
    let app = Router::new()
        .merge(record_router::<File>())
        .merge(record_router::<User>())
        .merge(record_router::<Project>())
        .merge(record_router::<Platform>())
        .merge(record_router::<ProjectRunner>())
        .merge(record_router::<PlatformRunner>())
        .merge(record_router::<Task>())
        .merge(record_router::<Assignment>())
        .merge(record_router::<Result>())
        .route("/register", post(routes::register))
        .route("/fetch_tasks", post(routes::fetch_tasks))
        .route("/submit_result/{id}", post(routes::submit_result))
        .route("/validate_fetch/{id}", get(routes::validate_fetch))
        .route("/validate_submit", post(routes::validate_submit))
        .route("/files", post(routes::create_file))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app).await.unwrap()
}

fn record_router<T>() -> Router<AppState>
where
    T: Record + Send + Unpin + Serialize + 'static,
    T::Filter: Select<Record = T> + Send + DeserializeOwned,
    Id<T>: Select<Record = T>,
{
    Router::new()
        .route(&format!("/{}", T::PATH), get(routes::get_all::<T>))
        .route(&format!("/{}/{{id}}", T::PATH), get(routes::get_one::<T>))
}
