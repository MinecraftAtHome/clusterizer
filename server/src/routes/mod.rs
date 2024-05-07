mod users;

use crate::app::App;
use axum::Router;
use std::sync::Arc;

pub fn router() -> Router<Arc<App>> {
    Router::new().nest("/users", users::router())
}
