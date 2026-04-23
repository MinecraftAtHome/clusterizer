use std::fs;

use args::{ClusterizerArgs, Commands};
use clap::Parser;
use clusterizer_api::client::ApiClient;
use clusterizer_client::result::ClientResult;
use clusterizer_common::requests::{CreateFileRequest, RegisterRequest};
use sha2::{Digest, Sha256};
use tracing::{debug, error, info};

mod args;
mod client;

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
        Commands::Run(mut args) => {
            fs::create_dir_all(&args.cache_dir)?;
            args.cache_dir = args.cache_dir.canonicalize()?;
            client::run(client, args).await?
        }
        Commands::CreateFile(args) => {
            debug!("Creating new file...");
            let bytes = reqwest::get(&args.url)
                .await?
                .error_for_status()?
                .bytes()
                .await?;
            let hash = Sha256::digest(bytes).0;

            let response = client
                .create_file(&CreateFileRequest {
                    url: args.url,
                    hash,
                })
                .await?;

            println!("{}", response);
            info!("Successfully created new file with ID: {}", response);
        }
    }

    Ok(())
}
