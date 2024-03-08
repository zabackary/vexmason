use std::{io::Write, path::Path, process::Stdio};

use anyhow::{bail, ensure, Context};

use octocrab::models::repos::Asset;
use reqwest::{Response, Url};
use tokio::{
    fs::{self, File},
    io::{AsyncWriteExt, BufWriter},
    process::Command,
};
use vexmason::{
    check_versions,
    installation_location::{
        get_installation_path, get_user_directory, VEXCOM_NAME, VEXCOM_OLD_NAME,
    },
};

const GITHUB_RELEASE_OWNER: &str = "zabackary";
const GITHUB_RELEASE_REPO: &str = "vexmason";

const VEXCOM_TMP_NAME: &str = "vexcom.tmp";

fn pause(msg: &str) {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    write!(stdout, "{}", msg).unwrap();
    stdout.flush().unwrap();

    let _ = stdin.read_line(&mut String::new()).unwrap();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subprocess = std::env::args().skip(1).next() == Some("--subprocess".to_string());
    match body(subprocess).await {
        Ok(_) => {
            println!("Installation has finished successfully.");
            if !subprocess {
                pause("Press ENTER to exit...");
            }
            Ok(())
        }
        Err(err) => {
            eprintln!("Something went wrong! Try re-running the installer. If that doesn't work, create a GitHub issue.\n{err:?}");
            if !subprocess {
                pause("\nPress ENTER to exit");
            }
            Err(err)
        }
    }
}

async fn body(subprocess: bool) -> anyhow::Result<()> {
    ensure!(!cfg!(target_os = "macos"), "At this time, MacOS is not supported. if you would like to support it, create a GitHub issue.");
    ensure!(cfg!(target_os = "windows"), "At this time, Windows is the only supported OS. Please create a GitHub issue if you would like to help support another.");

    check_versions::check_versions()?;

    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    println!(
        "Welcome to the vexmason installation wizard for {}.",
        version
    );
    if subprocess {
        println!("This installer was started as a subprocess.");
    }
    let octocrab = octocrab::instance();
    let repo = octocrab.repos(GITHUB_RELEASE_OWNER, GITHUB_RELEASE_REPO);
    let releases = repo.releases();
    println!("Checking for the latest updates...");
    let latest_release = releases.get_latest().await?;
    if latest_release.tag_name != version {
        println!("It looks like there's a newer version of the installer associated with the latest version {}.", latest_release.tag_name);
        println!("Downloading it to be used...");
        let mut installer_asset = None;
        for asset in latest_release.assets {
            if asset.name == "vexmason-installer.exe" {
                installer_asset = Some(asset);
            }
        }
        if let Some(installer_asset) = installer_asset {
            let filename = format!("vexmason-installer-{}.exe", latest_release.tag_name,);
            let dir = std::env::temp_dir();
            let path = dir.join(&filename);
            if path.try_exists()? {
                bail!("It seems like the file {} in {} already exists. If that's the installer for the new version, run that directly.", filename, dir.to_string_lossy());
            }
            install_bin(&filename, &installer_asset.browser_download_url, &dir).await?;
            println!("Downloaded.");
            println!("\n--- {} installer output ---\n", latest_release.tag_name);
            let mut child = Command::new(&path)
                .arg("--subprocess")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .with_context(|| "failed to spawn new installer")?;
            child.wait().await?;
            println!("\n--- finished {} installer ---", latest_release.tag_name);
            println!("Removing installer...");
            fs::remove_file(path).await?;
            println!("Removed.");
            return Ok(());
        } else {
            eprintln!(
                "Failed to find the latest installer, falling back to an outdated install of {}",
                version
            );
        }
    }

    let installation_directory = get_installation_path(std::env::current_exe().ok().as_deref())?;
    let already_installed = installation_directory.try_exists()?;
    if already_installed {
        // try to update
        println!("Already installed, attempting update");
        pause(&format!(
            "> Press ENTER to update installation at {}",
            installation_directory
                .to_str()
                .unwrap_or("unknown directory")
        ));
        if installation_directory.join("bin").try_exists()? {
            println!("> Uninstalling...");
            std::fs::remove_dir_all(installation_directory.join("bin"))?;
        } else {
            println!("> bin directory does not exist, skipping uninstall step");
        }
    } else {
        pause("Press ENTER to start installation...");
        println!(
            "> Installing vexmason into {}",
            installation_directory
                .to_str()
                .unwrap_or("unknown directory")
        );
    }
    println!("Downloading release metadata for {}...", version);
    let release = releases
        .get_by_tag(&version)
        .await
        .with_context(||anyhow::anyhow!("It seems like there's no release on GitHub yet for this version ({}). Maybe someone forgot to upload it.", version))?;
    println!("> Downloaded.");

    println!("Installing python-compiler...");
    std::fs::create_dir_all(installation_directory.join("bin"))?;
    let compiler_dir = installation_directory.join("lib").join("python-compiler");
    if compiler_dir.try_exists()? {
        println!("> python-compiler seems to be cloned already, pulling the latest changes...");
        pull_git_lib(&compiler_dir).await?;
    } else {
        println!("> Cloning python-compiler...");
        install_git_lib(
            &compiler_dir,
            "https://github.com/zabackary/python-compiler.git",
        )
        .await?;
    }

    println!("Installing vexcom hook...");
    let arch_dir_name = if cfg!(all(
        target_os = "windows",
        any(target_arch = "x86", target_arch = "x86_64")
    )) {
        "win32"
    } else if cfg!(all(target_os = "linux", target_arch = "arm")) {
        "linux-arm32"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "linux-arm64"
    } else if cfg!(all(
        target_os = "linux",
        any(target_arch = "x86", target_arch = "x86_64")
    )) {
        "linux-x86"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        bail!("unsupported operating system")
    };
    let vexcom_dir = get_user_directory(std::env::current_exe().ok().as_deref())?
        .join(".vscode")
        .join("extensions")
        .join("vexrobotics.vexcode-0.5.0")
        .join("resources")
        .join("tools")
        .join("vexcom")
        .join(arch_dir_name);
    if vexcom_dir.join(VEXCOM_OLD_NAME).exists() {
        println!("> vexcom hook already installed, skipping")
    } else {
        let mut found_asset: Option<&Asset> = None;
        for asset in &release.assets {
            if asset.name == "vexcom.exe" {
                found_asset = Some(asset);
            }
        }
        if let Some(asset) = found_asset {
            println!("> making a backup of {VEXCOM_NAME}...");
            fs::copy(vexcom_dir.join(VEXCOM_NAME), vexcom_dir.join("vexcom.bak")).await?;
            println!("> found artifact, downloading to {VEXCOM_TMP_NAME}...");
            let mut response = reqwest::get(asset.browser_download_url.clone())
                .await
                .with_context(|| "failed to fetch artifact")?
                .error_for_status()
                .with_context(|| "failed to fetch artifact")?;
            write_chunks(
                &mut response,
                &mut File::create(vexcom_dir.join(VEXCOM_TMP_NAME))
                    .await
                    .with_context(|| "failed to create tmp file to read download")?,
            )
            .await
            .with_context(|| "failed to copy download content")?;
            println!("> installing hook...");
            fs::rename(
                vexcom_dir.join(VEXCOM_NAME),
                vexcom_dir.join(VEXCOM_OLD_NAME),
            )
            .await?;
            fs::rename(
                vexcom_dir.join(VEXCOM_TMP_NAME),
                vexcom_dir.join(VEXCOM_NAME),
            )
            .await?;
            println!("> successfully installed.");
        } else {
            bail!("can't find vexcom.exe artifact in latest release");
        }
    }
    let bin_dir = installation_directory.join("bin");
    println!("Downloading binaries...");
    for asset in release.assets {
        if asset.name != "vexcom.exe" {
            install_bin(&asset.name, &asset.browser_download_url, &bin_dir).await?;
        }
    }
    Ok(())
}

async fn install_bin(filename: &str, url: &Url, dir: &Path) -> anyhow::Result<()> {
    println!("> downloading {}...", filename);
    let mut response = reqwest::get(url.clone())
        .await
        .with_context(|| "failed to fetch artifact")?
        .error_for_status()
        .with_context(|| "failed to fetch artifact")?;
    write_chunks(
        &mut response,
        &mut File::create(dir.join(filename))
            .await
            .with_context(|| "failed to create file to read download")?,
    )
    .await
    .with_context(|| "failed to copy download content")?;
    Ok(())
}

async fn install_git_lib(path: &Path, git_origin: &str) -> anyhow::Result<()> {
    let mut child = Command::new("git")
        .args(["clone", "--depth=1", git_origin])
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| "failed to spawn git")?;

    let exit_status = child
        .wait()
        .await
        .with_context(|| "failed to wait on child")?;
    if exit_status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "git clone failed with error code {}",
            exit_status.code().unwrap_or(-1)
        ))
    }
}

async fn pull_git_lib(path: &Path) -> anyhow::Result<()> {
    let mut child = Command::new("git")
        .arg("pull")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .current_dir(path)
        .spawn()
        .with_context(|| "failed to spawn git")?;

    let exit_status = child
        .wait()
        .await
        .with_context(|| "failed to wait on child")?;
    if exit_status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "git clone failed with error code {}",
            exit_status.code().unwrap_or(-1)
        ))
    }
}

async fn write_chunks(response: &mut Response, file: &mut fs::File) -> anyhow::Result<()> {
    let mut writer = BufWriter::new(file);
    while let Some(chunk) = response.chunk().await? {
        writer.write_all(&chunk).await?;
    }
    writer.flush().await?;
    Ok(())
}
