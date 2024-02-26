use std::{env, path::PathBuf, process::Stdio};

use anyhow::Context;

use chrono::Local;
use tokio::{
    fs,
    io::{stderr, AsyncWriteExt},
    process::Command,
};
use vexmason::{
    config::{resolved_config_from_entry_point, ResolvedConfig},
    installationlocation::VEXCOM_OLD_NAME,
    modify_args::{entry_point, modify_args, ModifyOptions},
    tee::tee,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let vexcom_location = Into::<PathBuf>::into(
        args.next()
            .ok_or(anyhow::anyhow!("can't read vexcom location"))?,
    );

    let mut args: Vec<String> = args.collect();

    match entry_point(&args).await {
        None => {
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

            let exit_status = child
                .wait()
                .await
                .with_context(|| "failed to wait on child")?;

            if exit_status.success() {
                Ok(())
            } else {
                std::process::exit(exit_status.code().unwrap_or(1));
            }
        }
        Some(entry_point) => {
            let config: ResolvedConfig = resolved_config_from_entry_point(entry_point).await?;
            let mut logs_destination = fs::File::create(config.log_output()).await?;
            logs_destination
                .write(
                    format!(
                        "vexmason log ({})\n",
                        Local::now().format("%a %d %b %Y, %I:%M%p")
                    )
                    .as_bytes(),
                )
                .await?;

            match modify_args(
                &mut args,
                &ModifyOptions {
                    name: &config.name,
                    description: &config.description,
                    compile_target_file: &config.build_output(),
                    compile_minify: config.minify,
                    compile_defines: &config.defines,
                },
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    logs_destination.write(format!("{e}\n").as_bytes()).await?;
                }
            }

            let mut child = Command::new(
                dunce::canonicalize(vexcom_location.with_file_name(VEXCOM_OLD_NAME))
                    .with_context(|| "failed to locate vexcom.old")?
                    .as_os_str(),
            )
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped())
            .stdin(Stdio::inherit())
            .spawn()
            .with_context(|| "failed to execute vexcom.old")?;

            let child_stderr_handle = tokio::task::spawn(tee(
                child
                    .stderr
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("failed to secure child stderr"))?,
                logs_destination,
                stderr(),
            ));

            let exit_status = child
                .wait()
                .await
                .with_context(|| "failed to wait on child")?;

            child_stderr_handle.await??;
            if exit_status.success() {
                Ok(())
            } else {
                std::process::exit(exit_status.code().unwrap_or(1));
            }
        }
    }
}
