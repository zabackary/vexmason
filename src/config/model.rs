use serde::{de::IgnoredAny, Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, mem, path::PathBuf};

pub const CURRENT_CONFIG_VERSION: semver::Version = semver::Version::new(1, 1, 0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigDefine {
    /// typed simple define
    SimpleTyped(ConfigDefineType),
    /// typed or not typed
    ExplicitTyped {
        default: ConfigDefineType,
        typed: bool,
    },
    /// a value restricted to a set of options
    Restricted {
        default: ConfigDefineType,
        options: Vec<ConfigDefineType>,
    },
}

impl ConfigDefine {
    pub fn validate_default(&self) -> bool {
        match self {
            ConfigDefine::Restricted { default, options } => options.contains(&default),
            _ => true,
        }
    }

    pub fn validate(&self, other: &ConfigDefineType) -> bool {
        match self {
            ConfigDefine::ExplicitTyped { default, typed } if *typed => {
                mem::discriminant(default) == mem::discriminant(other)
            }
            ConfigDefine::SimpleTyped(default) => {
                mem::discriminant(default) == mem::discriminant(other)
            }
            ConfigDefine::Restricted { options, .. } => options.contains(other),
            _ => true,
        }
    }
}

impl Into<ConfigDefineType> for ConfigDefine {
    fn into(self) -> ConfigDefineType {
        match self {
            ConfigDefine::SimpleTyped(default) => default,
            ConfigDefine::ExplicitTyped { default, .. } => default,
            ConfigDefine::Restricted { default, .. } => default,
        }
    }
}

impl Display for ConfigDefineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<String>::into(self.clone()))
    }
}

// make sure to update CURRENT_CONFIG_VERSION according to semver when updating
// this struct
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JsonConfigV1 {
    pub config_version: String,
    /// needs to be parsed for placeholders
    pub name: String,
    /// needs to be parsed for placeholders
    pub description: Option<String>,
    pub language: String,
    pub minify: Option<bool>,
    pub default_defines: Option<HashMap<String, ConfigDefine>>,
    pub entry_file: Option<String>,

    /// for the vscode extension
    #[serde(rename = "extension")]
    _extension: Option<IgnoredAny>,
}

// make sure to update CURRENT_CONFIG_VERSION according to semver when updating
// this struct
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JsonConfigV1Overrides {
    pub config_version: String,
    pub computer_name: Option<String>,
    pub defines_overrides: Option<HashMap<String, ConfigDefineType>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// needs to be parsed for placeholders
    pub name: String,
    /// needs to be parsed for placeholders
    pub description: String,
    pub language: String,
    pub defines: HashMap<String, ConfigDefineType>,
    pub project_root: PathBuf,
    pub minify: bool,
    pub entry_file: PathBuf,
}

impl ResolvedConfig {
    pub fn build_output(&self) -> PathBuf {
        self.project_root.join("build").join("compiled.py")
    }
}
