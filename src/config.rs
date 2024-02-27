mod model;
mod template;

use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context};
use model::{JsonConfigV1, JsonConfigV1Overrides};
use tokio::{
    fs,
    io::{self, AsyncReadExt},
};

pub use model::{ConfigDefineType, ResolvedConfig};

use self::template::evaluate_template;

const DEFAULT_DESCRIPTION: &str = "\u{1F530}{{ language::emoji }}{{ minify::emoji-pinch }}
compiled by vexmason
at {{ time/hour }}:{{ time/second }}
by {{ computer-name }}
| {{ defines::count }} defines:
{{ defines::list }}
";
const DEFAULT_MINIFY: bool = false;

const CONFIG_FILE: &str = "vexmason-config.json";
const CONFIG_OVERRIDES_FILE: &str = "vexmason-local-config.json";

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
    let config_overrides = config_overrides_from_file(config_overrides_path, config.config_version)
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

    // resolve defines
    let mut resolved_defines = config.default_defines.unwrap_or_else(|| HashMap::new());
    if let Some(defines_overrides) = config_overrides.defines_overrides {
        for (define_override, value) in defines_overrides {
            if resolved_defines.get(&define_override).is_some() {
                resolved_defines.insert(define_override, value);
            } else {
                bail!(
                "overrides file defines '{}' without a default value being present in the main config file",
                &define_override
            );
            }
        }
    }

    let computer_name = config_overrides
        .computer_name
        .as_ref()
        .map_or("unknown", |x| x);

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

    Ok(ResolvedConfig {
        config_version: config.config_version,
        defines: resolved_defines,
        description: resolved_description,
        language: config.language,
        name: resolved_name,
        project_root: project_root.to_path_buf(),
        minify,
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
    config_version: usize,
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
            config_version: config_version,
            computer_name: None,
            defines_overrides: None,
        }),
        Err(x) => Err(x.into()),
    }
}
