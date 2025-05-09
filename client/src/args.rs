use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "Clusterizer RS")]
#[command(version)]
#[command(about = "Crunching tasks since 2k20", long_about = None)]
pub struct Args {
    #[arg(long, short, default_value = ".")]
    pub data_dir: PathBuf,
    #[arg(long, short)]
    pub api_key: Option<String>,
    #[arg(long, short, default_value = "https://clusterizer.mcathome.dev")]
    pub server_url: String,
}
