use std::{
    env,
    error::Error,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use serde_json::Value;
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn Error>> {
    let mut temp_file: Option<NamedTempFile> = None;
    let mut args: Vec<String> = env::args().skip(1).collect();
    for (i, arg) in args.clone().iter().enumerate() {
        if arg == "--write" && i < args.len() - 1 {
            let file_name = &mut args[i + 1];
            let file_path = Path::new(&file_name);
            if let Some(extension) = file_path.extension() {
                if extension == "py" {
                    let transformed = transform_file(file_path)?;
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
                    let new_file_path = new_file.path().to_str().ok_or("failed!")?.to_string();
                    *file_name = new_file_path;
                    temp_file = Some(new_file);
                }
            }
        }
    }
    let mut child = Command::new("./vexcom.old")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .spawn()
        .expect("failed to execute vexcom.old");

    let exit_status = child.wait().expect("failed to wait on child");

    if let Some(file) = temp_file {
        file.keep()?;
    }

    // std::mem::drop(temp_file);

    if exit_status.success() {
        Ok(())
    } else {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
}

fn transform_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut transformer_child = Command::new("python")
        .args(vec![
            "-m",
            "python-compiler",
            "-i",
            path.canonicalize()?.to_str().ok_or("failed to transform")?,
            "--remove-imports",
            "vex",
            "--prelude",
            "from vex import *",
            "-j",
        ])
        .current_dir(std::fs::canonicalize("./lib/")?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .map_err(|_| "failed to execute vexcom.old")?;

    let exit_status = transformer_child.wait().expect("failed to wait on child");

    if exit_status.success() {
        let stdout = transformer_child.stdout.take().unwrap();
        let output_value: serde_json::Value = serde_json::from_reader(stdout)?;
        if let Some(Value::String(string)) = output_value.get("output") {
            // probably can't avoid clone
            Ok(string.to_string())
        } else {
            Err("transform failed: failed to read output".into())
        }
    } else {
        let stderr = transformer_child.stderr.take().unwrap();
        let output_value: serde_json::Value = serde_json::from_reader(stderr)?;
        if let (Some(Value::String(error_name)), Some(Value::String(error_msg))) =
            (output_value.get("name"), output_value.get("msg"))
        {
            Err(format!("transform failed: {error_name}: {error_msg}").into())
        } else {
            Err("transform failed: failed to read error".into())
        }
    }
}
