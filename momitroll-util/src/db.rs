use anyhow::{Result, anyhow};
use bson::doc;
use mongodb::Database;

pub async fn helthcheck(db: &Database) -> Result<()> {
    match db.run_command(doc! {"ping": 1}).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(e)),
    }
}
