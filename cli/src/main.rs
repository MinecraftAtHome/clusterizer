use clap::{Parser, Subcommand};
use clusterizer_client::client::Client;
use clusterizer_common::types::CreateUser;
use url::Url;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    url: Url,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    CreateUser {
        #[arg(long)]
        name: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = Client::new(args.url).unwrap();

    match args.command {
        Command::CreateUser { name } => {
            println!("{:?}", client.create_user(&CreateUser { name }).await);
        }
    }
}
