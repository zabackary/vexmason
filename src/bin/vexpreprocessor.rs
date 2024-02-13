use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::Context;
use tempfile::NamedTempFile;

use vex_python_preprocessor::{compilefile, installationlocation::VEXCOM_OLD_NAME};

fn main() -> anyhow::Result<()> {
    let mut temp_file: Option<NamedTempFile> = None;
    let mut args = env::args().skip(1);
    let vexcom_location = Into::<PathBuf>::into(
        args.next()
            .ok_or(anyhow::anyhow!("can't read vexcom location"))?,
    );
    let mut args: Vec<String> = args.collect();
    for (i, arg) in args.clone().iter().enumerate() {
        if arg == "--write" && i < args.len() - 1 {
            let file_name = &mut args[i + 1];
            let file_path = Path::new(&file_name);
            if let Some(extension) = file_path.extension() {
                if extension == "py" {
                    let transformed = compilefile::compile_file(file_path)?;
                    let mut new_file = tempfile::Builder::new()
                        .prefix("vexpythonpreprocessor-")
                        .suffix(&format!(
                            ".{}",
                            file_path
                                .extension()
                                .expect("filename doesn't have an extension")
                                .to_str()
                                .unwrap()
                        ))
                        .tempfile()?;
                    new_file.write_all(&transformed.into_bytes())?;
                    let new_file_path = new_file
                        .path()
                        .to_str()
                        .ok_or(anyhow::anyhow!("failed to parse path as utf-8"))?
                        .to_string();
                    *file_name = new_file_path;
                    temp_file = Some(new_file);
                }
            }
        }
    }

    let mut child = Command::new(
        dunce::canonicalize(vexcom_location.with_file_name(VEXCOM_OLD_NAME))
            .with_context(|| "failed to locate vexcom.old")?
            .as_os_str(),
    )
    .args(args)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .stdin(Stdio::inherit())
    .spawn()
    .with_context(|| "failed to execute vexcom.old")?;

    let exit_status = child.wait().with_context(|| "failed to wait on child")?;

    std::mem::drop(temp_file);

    if exit_status.success() {
        Ok(())
    } else {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
}
