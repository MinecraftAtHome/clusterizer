use sqlx::PgPool;
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub secret: Vec<u8>,
}
