use std::{fs::File, path::Path};

use zip::ZipArchive;

use crate::client::ClientError;

pub fn zip_extract(archive_file: &Path, target_dir: &Path) -> Result<(), ClientError> {
    let file = File::open(archive_file)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(target_dir)?;
    Ok(())
}
