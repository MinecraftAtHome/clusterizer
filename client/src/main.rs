use args::{Commands, GlobalArgs};
use clap::Parser;
use client::ClusterizerClient;
use clusterizer_common::messages::{RegisterRequest, RegisterResponse};
use result::ClientResult;

mod args;
mod client;
mod result;

pub async fn register(server_url: String, user_name: String) -> ClientResult<RegisterResponse> {
    Ok(clusterizer_api::Client::new(server_url, None)
        .register_user(&RegisterRequest { name: user_name })
        .await?)
}

#[tokio::main]
async fn main() -> ClientResult<()> {
    let global_args = GlobalArgs::parse();

    match global_args.command {
        Commands::Register(register_args) => {
            let register_response =
                register(global_args.server_url, register_args.username).await?;
            println!("Api Key: {} ", register_response.api_key);

            Ok(())
        }
        Commands::Run(run_args) => {
            Ok(ClusterizerClient::new(run_args, global_args.server_url)
                .run()
                .await?)
        }
    }
}
