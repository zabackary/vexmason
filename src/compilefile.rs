use std::{collections::HashMap, path::Path, process::Stdio};

use anyhow::{bail, Context};
use serde_json::Value;
use tokio::{io::AsyncReadExt, process::Command};

use crate::config::ConfigDefineType;

pub struct CompileFileOptions<'a> {
    pub input: &'a Path,
    pub output: Option<&'a Path>,
    pub minify: bool,
    pub defines: &'a HashMap<String, ConfigDefineType>,
}

pub async fn compile_file<'a>(options: &CompileFileOptions<'a>) -> anyhow::Result<Option<String>> {
    let mut lib_dir = std::env::current_exe()?;
    lib_dir.pop();
    lib_dir.pop();
    lib_dir.push("lib");
    let mut transformer_child = Command::new("python");
    transformer_child
        .args(["-m", "python-compiler", "--input"])
        .arg(dunce::canonicalize(options.input)?.as_os_str())
        .args([
            "--remove-imports",
            "vex",
            "--prelude",
            "from vex import *",
            "--json",
            "--export-dictionary-mode",
            "class_instance",
        ]);
    for (k, v) in options.defines {
        transformer_child.args(["--define-constant", k, &Into::<String>::into(v.clone())]);
    }
    if let Some(path) = options.output {
        transformer_child.arg("--output");
        transformer_child.arg(path.as_os_str());
    }
    transformer_child
        .current_dir(lib_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    let mut transformer_child = transformer_child
        .spawn()
        .with_context(|| "failed to execute run the python compiler")?;

    let exit_status = transformer_child
        .wait()
        .await
        .with_context(|| "failed to wait on child")?;

    if exit_status.success() {
        if options.output.is_none() {
            let mut stdout = transformer_child.stdout.take().unwrap();
            let mut read = Vec::new();
            stdout.read_to_end(&mut read).await?;
            let output_value: serde_json::Value = serde_json::from_slice(&read)?;
            if let Some(Value::String(string)) = output_value.get("output") {
                // probably can't avoid clone
                Ok(Some(string.to_string()))
            } else {
                bail!("transform failed: failed to read output")
            }
        } else {
            Ok(None)
        }
    } else {
        let mut stderr = transformer_child.stderr.take().unwrap();
        let mut read = Vec::new();
        stderr.read_to_end(&mut read).await?;
        let output_value: serde_json::Value = serde_json::from_slice(&read)?;
        if let (Some(Value::String(error_name)), Some(Value::String(error_msg))) =
            (output_value.get("name"), output_value.get("msg"))
        {
            bail!(format!("transform failed: {error_name}: {error_msg}"))
        } else {
            bail!("transform failed: failed to read error")
        }
    }
}
