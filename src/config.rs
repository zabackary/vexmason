mod model;
mod template;

use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context};
use log::{error, info, warn};
use model::{JsonConfigV1, JsonConfigV1Overrides};
use tokio::{
    fs,
    io::{self, AsyncReadExt},
};

pub use model::{ConfigDefineType, ResolvedConfig};

use self::{model::CURRENT_CONFIG_VERSION, template::evaluate_template};

const DEFAULT_DESCRIPTION: &str = "compiled by vexmason
at {{ time/hour }}:{{ time/minute }}
by {{ computer-name }} | {{ language::short }} | min: {{ minify::short }}
| {{ defines::count }} defines:
{{ defines::list }}
";
const DEFAULT_MINIFY: bool = false;

pub const CONFIG_FILE: &str = "vexmason-config.json";
pub const CONFIG_OVERRIDES_FILE: &str = "vexmason-local-config.json";

pub fn root(entry_point: &Path) -> Option<std::path::PathBuf> {
    let mut buf = entry_point.to_path_buf();
    if buf.is_file() {
        buf.pop();
    }
    while buf.is_dir() {
        if buf
            .join(".vscode")
            .join("vex_project_settings.json")
            .is_file()
            && buf.join(".vscode").join(CONFIG_FILE).is_file()
        {
            return Some(buf);
        }
        buf.pop();
    }
    None
}

async fn resolved_config_from_files(
    config_path: &Path,
    config_overrides_path: &Path,
    project_root: &Path,
) -> anyhow::Result<ResolvedConfig> {
    let config = config_from_file(config_path).await.with_context(|| {
        format!(
            "failed to read config file from {}",
            config_path
                .to_str()
                .expect("failed to decode config path to utf8")
        )
    })?;
    let config_overrides =
        config_overrides_from_file(config_overrides_path, &config.config_version)
            .await
            .with_context(|| {
                format!(
                    "failed to read config overrides file from {}",
                    config_path
                        .to_str()
                        .expect("failed to decode config path to utf8")
                )
            })?;
    if config.config_version != config_overrides.config_version {
        bail!("config and config overrides versions don't match");
    }
    let req = semver::VersionReq::parse(&format!("^{}", config.config_version)).with_context(||anyhow::anyhow!("failed to parse config version. it must be of the format 1.0 or 1.0.0 (e.g. 1.5 or 2.1 would be valid)"))?;
    if !req.matches(&CURRENT_CONFIG_VERSION) {
        error!(
            "your config requires version {}; this installation of vexmason supports up to {}",
            config.config_version, CURRENT_CONFIG_VERSION
        );
        bail!("the version specified in your config is not supported by your installation of vexmason. try updating.");
    }

    // resolve defines
    let default_defines = config.default_defines.unwrap_or_else(|| HashMap::new());
    let mut resolved_defines = HashMap::new();
    for (define, value) in &default_defines {
        if !value.validate_default() {
            bail!("the default define defined in {} did not pass its own type validation. check that `default_defines.{}.default` is contained in `default_defines.{}.options`.", CONFIG_FILE, define, define);
        }
        resolved_defines.insert(define.to_owned(), value.to_owned().into());
    }
    if let Some(defines_overrides) = config_overrides.defines_overrides {
        for (define_override, value) in defines_overrides {
            if resolved_defines.get(&define_override).is_some() {
                if let Some(default) = default_defines.get(&define_override) {
                    if default.validate(&value) {
                        info!(
                            "overriding define with local value: {} = {}",
                            define_override, value
                        );
                        resolved_defines.insert(define_override, value);
                    } else {
                        bail!(
                            "local config defines '{}' with the value '{}', but the default for that define in {} doesn't allow that type. make sure it's either included in `default_defines.{}.options` (if that exists) or the type is the same as the default if `default_defines.{}.typed` is `true`",
                            define_override, value, CONFIG_FILE, define_override, define_override
                        );
                    }
                }
            } else {
                warn!(
                    "local config defines '{}' without a default value being present in the main config file, ignoring",
                    define_override
                );
            }
        }
    }

    let computer_name = config_overrides.computer_name.as_ref().map_or_else(
        || {
            warn!("computer name not specified in config, defaulting to 'unknown'");
            "unknown"
        },
        |x| x,
    );

    let minify = config.minify.unwrap_or(DEFAULT_MINIFY);

    let resolved_name = evaluate_template(
        &config.name,
        computer_name,
        &config.language,
        minify,
        &resolved_defines,
    )
    .to_string();

    let resolved_description = evaluate_template(
        config
            .description
            .as_ref()
            .map_or(&DEFAULT_DESCRIPTION.replace('\n', " "), |x| x),
        computer_name,
        &config.language,
        minify,
        &resolved_defines,
    )
    .to_string();

    let resolved_entry_file = dunce::canonicalize(
        project_root.join(config.entry_file.unwrap_or("src/main.py".to_string())),
    )?;

    Ok(ResolvedConfig {
        defines: resolved_defines,
        description: resolved_description,
        language: config.language,
        name: resolved_name,
        project_root: project_root.to_path_buf(),
        minify,
        entry_file: resolved_entry_file,
    })
}

pub async fn resolved_config_from_root(root: &Path) -> anyhow::Result<ResolvedConfig> {
    let vscode = root.join(".vscode");
    resolved_config_from_files(
        &vscode.join(CONFIG_FILE),
        &vscode.join(CONFIG_OVERRIDES_FILE),
        &root,
    )
    .await
}

async fn config_from_file(path: &Path) -> anyhow::Result<JsonConfigV1> {
    let mut file = fs::File::open(path).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;
    let config =
        serde_json::from_str::<JsonConfigV1>(&content).with_context(|| "failed to parse config")?;
    return Ok(config);
}

async fn config_overrides_from_file(
    path: &Path,
    config_version: &str,
) -> anyhow::Result<JsonConfigV1Overrides> {
    match fs::File::open(path).await {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).await?;
            let config = serde_json::from_str::<JsonConfigV1Overrides>(&content)
                .with_context(|| "failed to parse config overrides")?;
            Ok(config)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(JsonConfigV1Overrides {
            config_version: config_version.to_owned(),
            computer_name: None,
            defines_overrides: None,
        }),
        Err(x) => Err(x.into()),
    }
}
