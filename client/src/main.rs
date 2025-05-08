mod types;

use clap::Parser;
use std::{
    ffi::OsStr,
    fs::{self},
    path::{Path, PathBuf},
    thread, time,
};
use types::{ClientError, ClusterizerClient};
use url::Url;
#[derive(Parser)]
#[command(name = "Clusterizer RS")]
#[command(version)]
#[command(about = "Crunching tasks since 2k20", long_about = None)]
struct Args {
    #[arg(long, short, default_value_t = String::from("."))]
    data_dir: String,
    #[arg(long, short)]
    api_key: Option<String>,
    #[arg(long, short, default_value_t = String::from("https://clusterizer.mcathome.dev"))]
    server_url: String,
}

/*
    MVP Client
    1. Create data folder x
    2. Create slots folder x
    3. Attempt to grab new assignments via the API x
    4. Grab task/project info using data given by assignment x
    5. Grab project binary url, download it x
    6. Create folder named by the assignment ID, place binary inside it. x
    7. Execute binary in working directory
    8. Create result instance containing the stdout, stderr, exit code, and assignment id x
    9. Submit result to api, receive result id x
    10. Repeat from #3 x
*/

async fn main_loop(client: &ClusterizerClient) -> Result<(), ClientError> {
    let task = client.fetch_tasks().await?;
    if task.is_empty() {
        eprintln!("No tasks found. Sleeping before attempting again.");
        thread::sleep(time::Duration::from_millis(15000));
        return Ok(());
    }
    let proj = client.get_project(task[0].project_id).await?;
    let proj_ver = client.get_project_project_version(proj.id).await?;

    println!("Task id: {}\t Task stdin: {}", task[0].id, task[0].stdin);
    println!("Projectid: {}\t Project name: {}", proj.id, proj.name);

    let slot_path: &Path = &client
        .data_dir
        .join("slots")
        .join(format!("{}", task[0].id));

    let _ = fs::create_dir_all(slot_path);

    let archive_url: Url = Url::parse(&proj_ver[0].archive_url)?;
    let archive_name: &str = match archive_url.path_segments().and_then(Iterator::last) {
        Some(url) => url,
        None => "error",
    };
    if archive_name == "error" {
        println!("Error: Could not retrieve archive name from url. {archive_url}");
    }
    let download_path = &slot_path.join(archive_name);

    let binary_name = client
        .get_program(download_path, slot_path, &proj_ver[0].archive_url)
        .await?;

    let prog_argc: Vec<&OsStr> = Vec::new();

    let result_data = client
        .run_program(slot_path, prog_argc, &binary_name)
        .await?;

    client.submit_task(task[0].id, &result_data).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let api_key = args.api_key;
    let data_dir: String = args.data_dir;
    let data_path = PathBuf::from(&data_dir);
    let server_url = args.server_url;
    let clusterizer_client = ClusterizerClient::new(api_key, server_url, data_path);
    println!("Using Data dir: {}", data_dir);
    fs::create_dir_all(format!("{}/slots", data_dir))?;

    loop {
        if let Err(err) = main_loop(&clusterizer_client).await {
            eprintln!("Error: {err}");
        }
    }
}
