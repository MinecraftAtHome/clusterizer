use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "Clusterizer RS")]
#[command(version)]
#[command(about = "Crunching tasks since 2k20", long_about = None)]
pub struct ClusterizerArgs {
    #[arg(long, short, default_value = "https://clusterizer.mcathome.dev")]
    pub server_url: String,
    #[arg(long, short)]
    pub api_key: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Supply name and optional server_url to register for clusterizer
    Register(RegisterArgs),
    /// Supply api_key, optional data_path, optional server_url, and a platform id to begin crunching on clusterizer
    Run(RunArgs),
}

#[derive(Debug, Args)]
pub struct RegisterArgs {
    #[arg(long, short)]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[arg(long, short, default_value = ".")]
    pub data_path: PathBuf,
    #[arg(long, short)]
    pub platform_id: i64,
}
