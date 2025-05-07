use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub secret: Vec<u8>,
    pub fetch_mutex: Arc<Mutex<()>>,
}
