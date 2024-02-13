/// Only code associated with running the main subprocess is here.
/// This will not be updated across versions.
use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::Context;
use vex_python_preprocessor::installationlocation::get_installation_path;

fn main() -> anyhow::Result<()> {
    let installation_path =
        get_installation_path(std::env::args().next().as_deref().map(Path::new))?;

    let location: String = dunce::canonicalize(
        std::env::current_exe().with_context(|| "failed to obtain current exe path")?,
    )
    .with_context(|| "failed to canonicalize")?
    .to_str()
    .ok_or(anyhow::anyhow!("path is not valid utf-8"))?
    .to_owned();

    let mut child = Command::new(installation_path.join("bin").join("vexpreprocessor"))
        .args([location].into_iter().chain(std::env::args().skip(1)))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .spawn()
        .with_context(|| "failed to spawn main preprocessor binary")?;

    let exit_status = child.wait().with_context(|| "failed to wait on child")?;

    if exit_status.success() {
        Ok(())
    } else {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
}
