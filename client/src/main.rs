use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::{thread, time};

use chrono::{DateTime, Utc};
use clap::Parser;
use clusterizer_api::Client as ClusterizerClient;
use clusterizer_common::types::{Assignment, Task};
use reqwest::Error;
use tokio::process::Command;
use zip::ZipArchive;
use zip::result::ZipResult;

#[derive(Parser)]
#[command(name = "Clusterizer RS")]
#[command(version = "0.0.1")]
#[command(about = "Crunching tasks since 2k20", long_about = None)]
struct Cli {
    #[arg(long, short,default_value_t = String::from("."))]
    datadir: String,
    #[arg(long, short)]
    apikey: String,
    #[arg(long, short,default_value_t = String::from("https://clusterizer.mcathome.dev"))]
    serverurl: String,
}

#[derive(Debug)]
enum MyError {
    Reqwest(reqwest::Error),
    Zip(zip::result::ZipError),
    Io(std::io::Error),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Reqwest(e) => write!(f, "Reqwest error: {}", e),
            MyError::Zip(e) => write!(f, "Zip error: {}", e),
            MyError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for MyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MyError::Reqwest(e) => Some(e),
            MyError::Zip(e) => Some(e),
            MyError::Io(e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for MyError {
    fn from(e: reqwest::Error) -> Self {
        MyError::Reqwest(e)
    }
}
impl From<zip::result::ZipError> for MyError {
    fn from(e: zip::result::ZipError) -> Self {
        MyError::Zip(e)
    }
}
impl From<std::io::Error> for MyError {
    fn from(e: std::io::Error) -> Self {
        MyError::Io(e)
    }
}
async fn zip_extract(archive_file: &Path, target_dir: &Path) -> Result<(), MyError> {
    let file = File::open(archive_file)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(target_dir);
    Ok(())
}

async fn get_program(
    download_path: &Path,
    slot_path: &Path,
    archive_url: String,
) -> Result<(), MyError> {
    let resp = reqwest::get(archive_url).await?;
    let body = resp.bytes().await?;
    let mut out = File::create(download_path)?;
    let mut reader = Cursor::new(body);
    io::copy(&mut reader, &mut out)?;
    zip_extract(&download_path, &slot_path);
    Ok(())
}

async fn run_program(
    task_id: i64,
    slot_path: &Path,
    prog_argc: &Vec<String>,
    prog_name: &OsStr,
) -> Result<clusterizer_common::messages::SubmitRequest, MyError> {
    println!("{}", prog_name.to_str().unwrap());

    let abs_path = fs::canonicalize(Path::new(&format!("{}/test-task.bin", slot_path.display())))?;
    let output = Command::new(abs_path)
        .args(prog_argc)
        .current_dir(slot_path)
        .output()
        .await?;

    let exit_code = output.status.code().unwrap_or(-100);

    let result_data = clusterizer_common::messages::SubmitRequest {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code,
    };
    Ok(result_data)
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut api_key = args.apikey;
    let data_dir = args.datadir;
    let server_url = args.serverurl;
    let mut clusterizer_client = ClusterizerClient::new(server_url, None);
    // Use api_key...
    if !api_key.is_empty() {
        println!("API Key: {}", api_key);
        clusterizer_client.set_api_key(api_key);
    } else {
        println!("No API key provided");
    }
    //Use data_dir...
    if !data_dir.is_empty() {
        println!("Using Data dir: {}", data_dir);
        fs::create_dir_all(format!("{}/slots", data_dir));
    } else {
        println!("No data dir provided. Defaulting to ./");
        fs::create_dir_all("./data/slots")?;
    }
    let sleep_duration = time::Duration::from_millis(15000);

    while true {
        let task = match clusterizer_client.fetch_tasks().await {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Failed to fetch assignments: {e}");
                continue;
            }
        };

        if task.len() == 0 {
            println!("No assignments available... Sleeping and trying again.");
            thread::sleep(sleep_duration);
            continue;
        }

        let proj = match clusterizer_client
            .get_project(task[0].project_id.clone())
            .await
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to get project: {e}");
                continue;
            }
        };

        let proj_ver = match clusterizer_client
            .get_project_version(task[0].project_id.clone())
            .await
        {
            Ok(pv) => pv,
            Err(e) => {
                eprintln!("Failed to get project version: {e}");
                continue;
            }
        };

        println!("Task id: {}\t Task stdin: {}", task[0].id, task[0].stdin);
        println!("Projectid: {}\t Project name: {}", proj.id, proj.name);

        let slot_str = format!("{}/slots/{}", data_dir, task[0].id);

        if let Err(e) = fs::create_dir_all(&slot_str) {
            eprintln!("Failed to create slot directory: {e}");
            continue;
        }

        let slot_path = Path::new(&slot_str);

        let binary_name = match proj_ver.archive_url.split('/').last() {
            Some(name) => name.to_string(),
            None => {
                eprintln!("Failed to extract binary name from archive_url");
                continue;
            }
        };

        let down_str = format!("{}/{}", slot_str, binary_name);

        let down_path = Path::new(&down_str);

        match get_program(down_path, slot_path.clone(), proj_ver.archive_url.clone()).await {
            Ok(bp) => bp,
            Err(e) => {
                eprintln!("Failed to get program: {e}");
                continue;
            }
        };

        let prog_argc = Vec::<String>::new();

        let result_data = match run_program(
            task[0].id,
            slot_path.clone(),
            &prog_argc,
            down_path.as_os_str(),
        )
        .await
        {
            Ok(rd) => rd,
            Err(e) => {
                eprintln!("Failed to run program: {e}");
                continue;
            }
        };

        let final_result = match clusterizer_client.submit_task(
            task[0].id,
            &result_data,
        )
        .await
        {
            Ok(fr) => fr,
            Err(e) => {
                eprintln!("Failed to submit result: {e}");
                continue;
            }
        };
    }
    Ok(())
}
