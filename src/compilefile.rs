use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{bail, Context};
use serde_json::Value;

pub fn compile_file(path: &Path) -> anyhow::Result<String> {
    let mut lib_dir = std::env::current_exe()?;
    lib_dir.pop();
    lib_dir.pop();
    lib_dir.push("lib");
    let mut transformer_child = Command::new("python")
        .args(["-m", "python-compiler", "--input"])
        .arg(dunce::canonicalize(path)?.as_os_str())
        .args([
            "--remove-imports",
            "vex",
            "--prelude",
            "from vex import *",
            "--json",
            "--export-dictionary-mode",
            "class_instance",
        ])
        .current_dir(lib_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| "failed to execute run the python compiler")?;

    let exit_status = transformer_child
        .wait()
        .with_context(|| "failed to wait on child")?;

    if exit_status.success() {
        let stdout = transformer_child.stdout.take().unwrap();
        let output_value: serde_json::Value = serde_json::from_reader(stdout)?;
        if let Some(Value::String(string)) = output_value.get("output") {
            // probably can't avoid clone
            Ok(string.to_string())
        } else {
            bail!("transform failed: failed to read output")
        }
    } else {
        let stderr = transformer_child.stderr.take().unwrap();
        let output_value: serde_json::Value = serde_json::from_reader(stderr)?;
        if let (Some(Value::String(error_name)), Some(Value::String(error_msg))) =
            (output_value.get("name"), output_value.get("msg"))
        {
            bail!(format!("transform failed: {error_name}: {error_msg}"))
        } else {
            bail!("transform failed: failed to read error")
        }
    }
}
