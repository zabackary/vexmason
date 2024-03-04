use std::{collections::HashMap, ffi::OsString, path::Path, process::Stdio, str::FromStr};

use anyhow::{bail, Context};
use log::{debug, error, info};
use serde_json::Value;
use tokio::{io::AsyncReadExt, process::Command};

use crate::config::ConfigDefineType;

pub struct CompileFileOptions<'a> {
    pub input: &'a Path,
    pub output: Option<&'a Path>,
    pub minify: bool,
    pub defines: &'a HashMap<String, ConfigDefineType>,
    pub app_data_location: &'a Path,
}

pub async fn compile_file<'a>(options: &CompileFileOptions<'a>) -> anyhow::Result<Option<String>> {
    let mut lib_dir = std::env::current_exe()?;
    lib_dir.pop();
    lib_dir.pop();
    lib_dir.push("lib");
    let mut transformer_child = Command::new("python");
    let mut args: Vec<OsString> = Vec::new();
    args.extend_from_slice(
        &["-m", "python-compiler", "--input"].map(|x| OsString::from_str(x).unwrap()),
    );
    args.push(dunce::canonicalize(options.input)?.as_os_str().to_owned());
    args.extend_from_slice(
        &[
            "--remove-imports",
            "vex",
            "--prelude",
            "from vex import *",
            "--json",
            "--export-dictionary-mode",
            "class_instance",
        ]
        .map(|x| OsString::from_str(x).unwrap()),
    );
    for (k, v) in options.defines {
        let value = Into::<String>::into(v.clone());
        args.extend_from_slice(
            &["--define-constant", k, &value].map(|x| OsString::from_str(x).unwrap()),
        );
    }
    if let Some(path) = options.output {
        args.push(OsString::from_str("--output").unwrap());
        args.push(path.as_os_str().to_owned());
    }

    info!("running python to compile entry file");
    debug!("args => {:?}", args);

    #[cfg(target_os = "windows")]
    {
        transformer_child.env("AppData", options.app_data_location);
    }
    transformer_child
        .args(args)
        .current_dir(dunce::canonicalize(lib_dir)?)
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
        info!("compiled successfully")
    } else {
        error!("failed to compile!")
    }

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
            bail!(format!("transform failed ({error_name}): {error_msg}"))
        }
        bail!("transform failed: failed to read error")
    }
}
