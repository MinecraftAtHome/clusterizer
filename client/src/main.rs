use args::Args;
use clap::Parser;
use client::ClusterizerClient;
use result::ClientResult;

mod args;
mod client;
mod result;

#[tokio::main]
async fn main() -> ClientResult<()> {
    ClusterizerClient::from(Args::parse()).run().await
}
