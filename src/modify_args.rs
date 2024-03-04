use std::path::Path;

use anyhow::Context;
use base64::Engine;

pub struct ModifyOptions<'a> {
    pub write_output: &'a Path,
    pub name: &'a str,
    /// must be converted to base64 before upload
    pub description: &'a str,
}

pub fn modify_args<'a>(args: &mut Vec<String>, options: &ModifyOptions<'a>) -> anyhow::Result<()> {
    for (i, flag) in args.clone().iter().take(args.len() - 1).enumerate() {
        // will never iterate over the last argument
        let argument = &mut args[i + 1];
        match flag.as_str() {
            "--write" => {
                *argument = options
                    .write_output
                    .to_str()
                    .with_context(|| anyhow::anyhow!("failed to convert output path to utf8"))?
                    .to_string();
            }
            "--name" => {
                *argument = options.name.to_string();
            }
            "--description" => {
                *argument = base64::prelude::BASE64_STANDARD.encode(options.description);
            }
            _ => (),
        }
    }
    Ok(())
}

pub fn has_write<'a>(args: &Vec<String>) -> bool {
    for flag in args {
        if flag == "--write" {
            return true;
        }
    }
    false
}

pub fn entry_point(args: &Vec<String>) -> Option<&Path> {
    for (i, flag) in args.iter().take(args.len() - 1).enumerate() {
        // will never iterate over the last argument
        if flag == "--write" {
            return Some(Path::new(&args[i + 1]));
        }
    }
    None
}
