use anyhow::{Result, anyhow};
use std::path::Path;

pub fn check_file<P: AsRef<Path>>(path: P) -> Result<()> {
    if !Path::new(path.as_ref()).exists() {
        Err(anyhow!("file {} doesn't exist", path.as_ref().display()))
    } else {
        Ok(())
    }
}
