use std::{collections::HashMap, path::Path};

use anyhow::Context;
use base64::Engine;
use tokio::io::AsyncWrite;

use crate::{
    compile_file::{self, CompileFileOptions},
    config::ConfigDefineType,
};

pub struct ModifyOptions<'a> {
    pub compile_target_file: &'a Path,
    pub compile_minify: bool,
    pub compile_defines: &'a HashMap<String, ConfigDefineType>,
    pub name: &'a str,
    /// must be converted to base64 before upload
    pub description: &'a str,
    pub app_data_location: &'a Path,
}

pub async fn modify_args<'a>(
    args: &mut Vec<String>,
    options: &ModifyOptions<'a>,
    log_file: &mut (impl AsyncWrite + std::marker::Unpin),
) -> anyhow::Result<()> {
    for (i, flag) in args.clone().iter().take(args.len() - 1).enumerate() {
        // will never iterate over the last argument
        let argument = &mut args[i + 1];
        match flag.as_str() {
            "--write" => {
                modify_write(
                    argument,
                    options.compile_target_file,
                    options.compile_minify,
                    options.compile_defines,
                    options.app_data_location,
                    log_file,
                )
                .await?
            }
            "--name" => modify_name(argument, options.name),
            "--description" => modify_description(argument, options.description),
            _ => (),
        }
    }
    Ok(())
}

pub async fn entry_point(args: &Vec<String>) -> Option<&Path> {
    for (i, flag) in args.iter().take(args.len() - 1).enumerate() {
        // will never iterate over the last argument
        if flag == "--write" {
            return Some(Path::new(&args[i + 1]));
        }
    }
    None
}

async fn modify_write(
    argument: &mut String,
    compile_target_file: &Path,
    minify: bool,
    defines: &HashMap<String, ConfigDefineType>,
    app_data_location: &Path,
    log_file: &mut (impl AsyncWrite + std::marker::Unpin),
) -> anyhow::Result<()> {
    let file_path = Path::new(&argument);
    if let Some(extension) = file_path.extension() {
        if extension == "py" {
            compile_file::compile_file(
                &CompileFileOptions {
                    input: file_path,
                    output: Some(compile_target_file), // ask python to do the write for us
                    minify,
                    defines,
                    app_data_location,
                },
                log_file,
            )
            .await?;
            let target_file_str = compile_target_file
                .to_str()
                .with_context(|| "couldn't decode output file path as utf8")?;
            *argument = target_file_str.to_string();
        }
    }
    Ok(())
}

fn modify_name(argument: &mut String, new_name: &str) {
    *argument = new_name.to_string();
}

fn modify_description(argument: &mut String, new_description: &str) {
    *argument = base64::prelude::BASE64_STANDARD.encode(new_description);
}
