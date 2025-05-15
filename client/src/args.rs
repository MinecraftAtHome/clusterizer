use std::path::PathBuf;

use clap::{
    Args, Parser, Subcommand,
    builder::{OsStr, Resettable},
};
use clusterizer_common::{id::Id, types::Platform};

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
    /// Supply api_key, optional cache_dir, optional server_url, and a platform id to begin crunching on clusterizer
    Run(RunArgs),
}

#[derive(Debug, Args)]
pub struct RegisterArgs {
    #[arg(long, short)]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[arg(long, short, default_value = cache_dir())]
    pub cache_dir: PathBuf,
    #[arg(long, short)]
    pub platform_id: Id<Platform>,
}

fn cache_dir() -> Resettable<OsStr> {
    dirs::cache_dir()
        .map(|path| path.join("clusterizer").into_os_string().into())
        .into()
}
