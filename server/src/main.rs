mod auth;
mod result;
mod routes;
mod state;

use std::env;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let database_url = env::var("DATABASE_URL").unwrap();
    let secret = env::var("CLUSTERIZER_SECRET").unwrap();

    let state = AppState {
        pool: PgPool::connect(&database_url).await.unwrap(),
        secret: secret.into_bytes(),
    };

    let app = Router::new()
        .route("/users", get(routes::users::get_all))
        .route("/users/{id}", get(routes::users::get_one))
        .route("/users/profile", get(routes::users::profile))
        .route("/users/register", post(routes::users::register))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
