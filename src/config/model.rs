use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigDefineType {
    String(String),
    Number(f32),
    Boolean(bool),
}

impl Into<String> for ConfigDefineType {
    fn into(self) -> String {
        match self {
            ConfigDefineType::Boolean(a) => a.to_string(),
            ConfigDefineType::Number(a) => a.to_string(),
            ConfigDefineType::String(a) => a,
        }
    }
}

impl Display for ConfigDefineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<String>::into(self.clone()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct JsonConfigV1 {
    pub config_version: usize,
    /// needs to be parsed for placeholders
    pub name: String,
    /// needs to be parsed for placeholders
    pub description: Option<String>,
    pub language: String,
    pub minify: Option<bool>,
    pub default_defines: Option<HashMap<String, ConfigDefineType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct JsonConfigV1Overrides {
    pub config_version: usize,
    pub computer_name: Option<String>,
    pub defines_overrides: Option<HashMap<String, ConfigDefineType>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub config_version: usize,
    /// needs to be parsed for placeholders
    pub name: String,
    /// needs to be parsed for placeholders
    pub description: String,
    pub language: String,
    pub defines: HashMap<String, ConfigDefineType>,
    pub project_root: PathBuf,
    pub minify: bool,
}

impl ResolvedConfig {
    pub fn build_output(&self) -> PathBuf {
        self.project_root.join("build").join("compiled.py")
    }
}
