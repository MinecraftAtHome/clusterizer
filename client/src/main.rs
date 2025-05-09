use args::Args;
use clap::Parser;
use client::{ClientError, ClusterizerClient};

mod args;
mod client;
mod util;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    ClusterizerClient::from(Args::parse()).run().await
}
