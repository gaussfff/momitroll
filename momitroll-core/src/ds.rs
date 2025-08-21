use bson::{DateTime, Document};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Default)]
pub struct Migration {
    pub name: String,
    pub applied_at: Option<DateTime>,
    pub status: MigrationStatus,
    pub description: Option<String>,
}

impl Migration {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum MigrationStatus {
    #[serde(rename = "pending")]
    #[default]
    Pending,
    #[serde(rename = "applied")]
    Applied,
}

impl MigrationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MigrationStatus::Pending => "pending",
            MigrationStatus::Applied => "applied",
        }
    }
}

impl fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationStatus::Pending => write!(f, "{}", "pending".yellow()),
            MigrationStatus::Applied => write!(f, "{}", "applied".green()),
        }
    }
}

pub struct MigrationContent {
    pub description: String,
    pub commands: Vec<Document>,
}

impl<'de> Deserialize<'de> for MigrationContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use bson::Bson;
        use serde::de::Error;
        use serde_json::{Map, Value};

        let map: Map<String, Value> = Deserialize::deserialize(deserializer)?;

        let description = map
            .get("description")
            .ok_or_else(|| Error::missing_field("description"))?
            .as_str()
            .ok_or_else(|| Error::custom("description must be a string"))?
            .to_string();

        let command_objects = map
            .get("commands")
            .ok_or_else(|| Error::missing_field("commands"))?
            .as_array()
            .ok_or_else(|| Error::custom("commands must be an array"))?;

        let mut commands = vec![];

        for command_obj in command_objects {
            let command = Bson::try_from(
                command_obj
                    .as_object()
                    .ok_or_else(|| Error::custom("each command must be an object"))?
                    .clone(),
            )
            .map_err(|_| Error::custom("failed to convert command to Bson"))?;

            match command.as_document() {
                Some(doc) => commands.push(doc.clone()),
                None => return Err(Error::custom("each command must be an object")),
            }
        }

        Ok(MigrationContent {
            description,
            commands,
        })
    }
}
