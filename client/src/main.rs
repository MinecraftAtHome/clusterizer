use args::{Commands, GlobalArgs};
use clap::Parser;
use client::ClusterizerClient;
use result::{ClientError, ClientResult};

mod args;
mod client;
mod result;

#[tokio::main]
async fn main() -> ClientResult<()> {
    let global_args = GlobalArgs::parse();

    match global_args.command {
        Some(Commands::Register(register_args)) => {
            let register_response =
                ClusterizerClient::register(global_args.server_url, register_args.username).await;
            match register_response {
                Ok(register_content) => {
                    println!("Api Key: {} ", register_content.api_key);
                }
                Err(_) => {
                    return Err(ClientError::RegistrationError);
                }
            }
            Ok(())
        }
        Some(Commands::Run(run_args)) => {
            Ok(ClusterizerClient::new(run_args, global_args.server_url)
                .await
                .run()
                .await?)
        }
        None => Err(ClientError::NoCommand),
    }
}
