use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Clusterizer RS")]
#[command(version)]
#[command(about = "Crunching tasks since 2k20", long_about = None)]
pub struct GlobalArgs {
    #[arg(long, short, default_value = "https://clusterizer.mcathome.dev")]
    pub server_url: String,
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Supply user_name and optional server_url to register for clusterizer
    Register(RegisterArgs),
    /// Supply api_key, optional data_path, optional server_url, and a platform id to begin crunching on clusterizer
    Run(RunArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[clap(long, short, default_value = ".")]
    pub data_path: PathBuf,
    #[clap(long, short)]
    pub api_key: Option<String>,
    #[clap(long, short)]
    pub platform_id: i64,
}

#[derive(Debug, Args)]
pub struct RegisterArgs {
    #[clap(long, short)]
    pub username: String,
}
