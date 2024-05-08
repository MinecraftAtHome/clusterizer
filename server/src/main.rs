mod app;
mod error;
mod routes;

use app::App;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let config = env::var("CLUSTERIZER_DATABASE").unwrap();
    let (client, connection) = tokio_postgres::connect(&config, NoTls).await.unwrap();

    tokio::spawn(async move {
        connection.await.unwrap();
    });

    let state = Arc::new(App::new(client));
    let app = routes::router().with_state(state);
    let address = env::var("CLUSTERIZER_ADDRESS").unwrap();
    let listener = TcpListener::bind(&address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
