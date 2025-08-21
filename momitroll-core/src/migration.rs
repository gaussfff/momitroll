use anyhow::Result;
use bson::{Document, doc};
use futures::stream::StreamExt;
use mongodb::Database;
use std::{
    fs::{File, create_dir_all},
    path::Path,
};
use tracing::{info, warn};

use crate::ds::*;
use momitroll_config::Config;
use momitroll_util::{db::helthcheck, file::check_file};

pub struct MigrationController {
    db: Database,
    config: Config,
}

impl MigrationController {
    pub async fn new(config: Config) -> Result<Self> {
        use mongodb::Client;

        let (username, password) = config.creds_env_vars.get_creds()?;

        let db = Client::with_uri_str(&format!(
            "mongodb://{}:{}@{}:{}/?authSource=admin",
            username, password, config.db.host, config.db.port
        ))
        .await?
        .database(&config.db.name);

        helthcheck(&db).await?;

        Ok(Self { db, config })
    }

    pub async fn init(&self) -> Result<()> {
        let migration_dir = &self.config.migration.dir;
        let collection_name = self.config.migration.coll_name();

        if !Path::new(&migration_dir).exists() {
            create_dir_all(&self.config.migration.dir)?;

            info!("migration directory created: {migration_dir}");
        } else {
            warn!("migration directory already exists: {migration_dir}");
        }

        if !self.is_exist_collection(&collection_name).await? {
            self.db.run_command(doc! {
                "create": self.config.migration.coll_name(),
                "validator": {
                    "$jsonSchema": {
                        "bsonType": "object",
                        "required": ["name", "status"],
                        "properties": {
                            "name": {
                                "bsonType": "string",
                                "description": "name of migration"
                            },
                            "checksum": {
                                "bsonType": ["int", "null"],
                                "description": "checksum of migration file, used to check if migration file was changed"
                            },
                            "applied_at": {
                                "bsonType": ["date", "null"],
                                "description": "date of when migration was applied"
                            },
                            "status": {
                                "bsonType": "string",
                                "enum": ["pending", "applied"],
                                "description": "status of migration"
                            },
                            "description": {
                                "bsonType": ["string", "null"],
                                "description": "description of migration"
                            }
                        }
                    }
                }
            }).await?;

            info!("migrations collection created: {collection_name}");
        } else {
            warn!("migrations collection already exists: {collection_name}");
        }

        Ok(())
    }

    pub async fn create(&self, name: &str) -> Result<()> {
        use chrono::Utc;

        self.check_migration_collection().await?;

        let name = format!("{}_{name}", Utc::now().timestamp());
        let dir_path = format!("{}/{name}", self.config.migration.dir);

        create_dir_all(&dir_path)?;
        Self::init_migration_file(format!("{dir_path}/{name}_up.json"))?;
        Self::init_migration_file(format!("{dir_path}/{name}_down.json"))?;

        self.db
            .collection::<Migration>(&self.config.migration.coll_name())
            .insert_one(Migration::new(name.clone()))
            .await?;

        info!("migration created: {name}");

        Ok(())
    }

    pub async fn up(&self) -> Result<()> {
        use bson::DateTime;
        use chrono::Local;

        self.check_migration_collection().await?;

        let collection = self
            .db
            .collection::<Migration>(&self.config.migration.coll_name());
        let mut res = collection.find(doc! {}).sort(doc! { "name": 1 }).await?;

        while let Some(migration) = res.next().await {
            let migration = migration?;
            let migration_name = migration.name.clone();
            let file_path = format!(
                "{}/{migration_name}/{migration_name}_up.json",
                self.config.migration.dir
            );

            check_file(&file_path)?;

            let migration_description = self.apply_commands(&file_path).await?;

            collection
                .update_one(
                    doc! { "name": migration_name },
                    doc! { "$set": {
                        "applied_at": DateTime::from_chrono(Local::now()),
                        "status": "applied",
                        "description": migration_description,
                    } },
                )
                .await?;

            info!("applied migration: {}", migration.name);
        }

        Ok(())
    }

    pub async fn down(&self) -> Result<()> {
        use bson::Bson;

        self.check_migration_collection().await?;

        match self
            .find_one(MigrationStatus::Applied, doc! { "applied_at": -1 })
            .await?
        {
            Some(migration) => {
                // TODO: question about moving?
                let migration_name = migration.name.clone();
                let file_path = format!(
                    "{}/{migration_name}/{migration_name}_down.json",
                    self.config.migration.dir
                );

                check_file(&file_path)?;

                let _ = self.apply_commands(&file_path).await?;

                self.db
                    .collection::<Migration>(&self.config.migration.coll_name())
                    .update_one(
                        doc! { "name": migration_name },
                        doc! { "$set": {
                            "applied_at": Bson::Null,
                            "status": "pending"
                        } },
                    )
                    .await?;

                info!("rollbacked migration: {}", migration.name);
            }
            None => {
                warn!("can't find last applyed migration")
            }
        }

        Ok(())
    }

    pub async fn status(&self) -> Result<()> {
        use colored::Colorize;

        self.check_migration_collection().await?;

        let collection = self
            .db
            .collection::<Migration>(&self.config.migration.coll_name());

        if collection.count_documents(doc! {}).await? == 0 {
            warn!("no migrations found");
            return Ok(());
        }

        let mut res = collection.find(doc! {}).sort(doc! { "name": -1 }).await?;

        while let Some(migration) = res.next().await {
            let migration = migration?;
            let date = match migration.applied_at {
                Some(dt) => dt.to_string().green(),
                None => "<not applied>".to_string().red(),
            };

            println!(
                "name: {}, applied at: {}, status: {}, description: {}",
                migration.name.blue(),
                date,
                migration.status,
                migration
                    .description
                    .unwrap_or_else(|| "<empty>".to_string())
                    .cyan(),
            );
        }

        Ok(())
    }

    pub async fn drop(&self) -> Result<()> {
        use std::fs::remove_dir_all;

        self.check_migration_collection().await?;

        match self
            .find_one(MigrationStatus::Pending, doc! { "name": -1 })
            .await?
        {
            Some(migration) => {
                self.db
                    .collection::<Migration>(&self.config.migration.coll_name())
                    .delete_one(doc! { "name": migration.name.clone() })
                    .await?;

                remove_dir_all(format!("{}/{}", self.config.migration.dir, migration.name))?;

                info!("dropped migration: {}", migration.name);
            }
            None => {
                warn!("no pending migrations found");
            }
        }

        Ok(())
    }

    fn init_migration_file<P: AsRef<Path>>(path: P) -> Result<()> {
        use std::io::Write;

        let mut file = File::create(&path)?;
        write!(
            &mut file,
            r#"{{
    "description\": "TODO: describe which changes will do this migration",
    "commands": [
                    
    ]
}}
            "#
        )?;

        Ok(())
    }

    async fn find_one(
        &self,
        status: MigrationStatus,
        sort_clause: Document,
    ) -> Result<Option<Migration>> {
        use mongodb::options::FindOneOptions;

        Ok(self
            .db
            .collection::<Migration>(&self.config.migration.coll_name())
            .find_one(doc! { "status": status.as_str() })
            .with_options(FindOneOptions::builder().sort(sort_clause).build())
            .await?)
    }

    async fn apply_commands<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        use serde_json::from_str;

        let migration_content = from_str::<MigrationContent>(&std::fs::read_to_string(path)?)?;
        for command in migration_content.commands {
            self.db.run_command(command).await?;
        }

        Ok(migration_content.description)
    }

    async fn check_migration_collection(&self) -> Result<()> {
        if !self
            .is_exist_collection(&self.config.migration.coll_name())
            .await?
        {
            return Err(anyhow::anyhow!("migrations collection does not exist"));
        }
        Ok(())
    }

    async fn is_exist_collection(&self, name: &str) -> Result<bool> {
        Ok(self
            .db
            .list_collection_names()
            .await?
            .into_iter()
            .any(|n| n == name))
    }
}
