use anyhow::ensure;

use vex_python_preprocessor::{checkversions, installationlocation::get_installation_path};

fn main() -> anyhow::Result<()> {
    ensure!(!cfg!(target_os = "macos"), "at this time, MacOS is not supported. if you would like to support it, create a GitHub issue.");

    checkversions::check_versions()?;
    let installation_directory = get_installation_path(None)?;
    let already_installed = installation_directory.try_exists()?;
    if already_installed {
        // try to update
        println!("already installed, attempting update");
        println!("uninstalling...");
    }
    Ok(())
}
