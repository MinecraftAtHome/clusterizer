use args::{ClusterizerArgs, Commands};
use clap::Parser;
use client::ClusterizerClient;
use clusterizer_api::client::ApiClient;
use clusterizer_common::requests::RegisterRequest;
use env_logger::Env;
use log::{debug, error};
use result::ClientResult;

mod args;
mod client;
mod result;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

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
        Commands::Run(args) => ClusterizerClient::new(client, args).run().await?,
    }

    Ok(())
}
