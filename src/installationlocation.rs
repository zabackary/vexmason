use anyhow::{bail, ensure, Context};
use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
    str::FromStr,
};

#[cfg(target_os = "windows")]
pub const VEXCOM_OLD_NAME: &str = "vexcom.old.exe";
#[cfg(target_os = "macos")]
pub const VEXCOM_OLD_NAME: &str = "vexcom.old";
#[cfg(target_os = "linux")]
pub const VEXCOM_OLD_NAME: &str = "vexcom.old";

#[cfg(target_os = "windows")]
pub const VEXCOM_NAME: &str = "vexcom.exe";
#[cfg(target_os = "macos")]
pub const VEXCOM_NAME: &str = "vexcom";
#[cfg(target_os = "linux")]
pub const VEXCOM_NAME: &str = "vexcom";

const INSTALLATION_DIRECTORY: &str = "vexmason";

/// Gets the path of the current installation.
///
/// If it can't be determined normally (i.e., using the home directory and/or
/// environment variables), uses reference_path to find it.
pub fn get_installation_path(reference_path: Option<&Path>) -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let local_app_data: PathBuf = match std::env::var("LOCALAPPDATA") {
            Ok(dir) => dir.into(),
            Err(err) => {
                if let Some(path) = reference_path {
                    let path = dunce::canonicalize(path)?;
                    let components: Vec<Component> = path.components().collect();
                    let root = components
                        .iter()
                        .position(|&component| component == Component::RootDir)
                        .ok_or(anyhow::anyhow!(
                            "path is relative so the user directory cannot be determined"
                        ))?;
                    ensure!(
                        components.len() > root + 2,
                        "path is not long enough to determine user directory"
                    );
                    ensure!(
                        // TODO: ensure this works for non-English Windows
                        // installations
                        components[root + 1] == Component::Normal(&OsString::from_str("Users")?),
                        "path is a system path and does not contain /Users/"
                    );
                    path.iter()
                        .take(root + 3)
                        .collect::<PathBuf>()
                        .join("AppData")
                        .join("Local")
                } else {
                    Err(err).with_context(|| {
                        "can't get installation path: couldn't recover from missing LOCALAPPDATA"
                    })?
                }
            }
        };
        Ok(local_app_data.join(INSTALLATION_DIRECTORY))
    } else if cfg!(target_os = "linux") {
        Ok(PathBuf::from_str("~/.local/bin")?.join(INSTALLATION_DIRECTORY))
    } else {
        bail!("unsupported operating system")
    }
}

pub fn get_user_directory(reference_path: Option<&Path>) -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        Ok(match std::env::var("USERPROFILE") {
            Ok(dir) => dir.into(),
            Err(err) => {
                if let Some(path) = reference_path {
                    let path = dunce::canonicalize(path)?;
                    let components: Vec<Component> = path.components().collect();
                    let root = components
                        .iter()
                        .position(|&component| component == Component::RootDir)
                        .ok_or(anyhow::anyhow!(
                            "path is relative so the user directory cannot be determined"
                        ))?;
                    ensure!(
                        components.len() > root + 2,
                        "path is not long enough to determine user directory"
                    );
                    ensure!(
                        // TODO: ensure this works for non-English Windows
                        // installations
                        components[root + 1] == Component::Normal(&OsString::from_str("Users")?),
                        "path is a system path and does not contain /Users/"
                    );
                    path.iter().take(root + 3).collect::<PathBuf>()
                } else {
                    Err(err).with_context(|| {
                        "can't get installation path: couldn't recover from missing USERPROFILE"
                    })?
                }
            }
        })
    } else if cfg!(target_os = "linux") {
        Ok(PathBuf::from_str("~")?)
    } else {
        bail!("unsupported operating system")
    }
}
