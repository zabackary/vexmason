use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{ensure, Context};

use vexmason::{checkversions, installationlocation::get_installation_path};

fn main() -> anyhow::Result<()> {
    ensure!(!cfg!(target_os = "macos"), "at this time, MacOS is not supported. if you would like to support it, create a GitHub issue.");

    checkversions::check_versions()?;
    let installation_directory = get_installation_path(std::env::current_exe().ok().as_deref())?;
    let already_installed = installation_directory.try_exists()?;
    if already_installed {
        // try to update
        println!("already installed, attempting update");
        if installation_directory.join("bin").try_exists()? {
            println!("uninstalling...");
            std::fs::remove_dir_all(installation_directory.join("bin"))?;
        } else {
            println!("bin directory does not exist, skipping uninstall step");
        }
    }
    std::fs::create_dir_all(installation_directory.join("bin"))?;
    let compiler_dir = installation_directory.join("lib").join("python-compiler");
    if compiler_dir.try_exists()? {
        println!("python-compiler seems to be cloned already, pulling the latest changes...");
        pull_git_lib(&compiler_dir)?;
    } else {
        println!("cloning python-compiler...");
        install_git_lib(
            &compiler_dir,
            "https://github.com/zabackary/python-compiler.git",
        )?;
    }
    Ok(())
}

fn install_git_lib(path: &Path, git_origin: &str) -> anyhow::Result<()> {
    let mut child = Command::new("git")
        .args([
            "clone",
            git_origin,
            path.to_str()
                .ok_or(anyhow::anyhow!("path is not valid utf-8"))?,
        ])
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| "failed to spawn git")?;

    let exit_status = child.wait().with_context(|| "failed to wait on child")?;
    if exit_status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "git clone failed with error code {}",
            exit_status.code().unwrap_or(-1)
        ))
    }
}

fn pull_git_lib(path: &Path) -> anyhow::Result<()> {
    let mut child = Command::new("git")
        .arg("pull")
        .stderr(Stdio::inherit())
        .current_dir(path)
        .spawn()
        .with_context(|| "failed to spawn git")?;

    let exit_status = child.wait().with_context(|| "failed to wait on child")?;
    if exit_status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "git clone failed with error code {}",
            exit_status.code().unwrap_or(-1)
        ))
    }
}
