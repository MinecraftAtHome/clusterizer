use std::sync::Arc;

use args::{ClusterizerArgs, Commands};
use clap::Parser;
use client::ClusterizerClient;
use clusterizer_api::client::ApiClient;
use clusterizer_common::requests::RegisterRequest;
use result::ClientResult;
use tracing::{debug, error};

mod args;
mod client;
mod result;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Err(err) = run().await {
        error!("Error: {err}.");
    }
}

async fn run() -> ClientResult<()> {
    let args = ClusterizerArgs::parse();
    let client = ApiClient::new(args.server_url, args.api_key);

    match args.command {
        Commands::Register(args) => {
            debug!("Registering...");

            let response = client
                .register(&RegisterRequest { name: args.name })
                .await?;

            println!("{}", response.api_key);
        }
        Commands::Run(args) => Arc::new(ClusterizerClient::new(client, args)).run().await?,
    }

    Ok(())
}
