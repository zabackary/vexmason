use std::{env, path::PathBuf, process::Stdio};

use anyhow::Context;

use chrono::Local;
use tokio::{
    fs,
    io::{stderr, AsyncWriteExt},
    process::Command,
};
use vexmason::{
    config::{resolved_config_from_root, root},
    installation_location::{self, VEXCOM_OLD_NAME},
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
    let user_directory = installation_location::get_user_directory(Some(&vexcom_location))?;

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
            let root = root(entry_point).with_context(||"Failed to resolve config! Make sure the project has a vex_project_settings.json and a {} present.")?;
            let build = root.join("build");
            if !build.exists() {
                fs::create_dir(root.join("build"))
                    .await
                    .with_context(|| "couldn't create output dir")?;
            }
            let mut logs_file = fs::File::create(build.join("vexmason-log.txt"))
                .await
                .with_context(|| "couldn't create log file")?;
            logs_file
                .write(
                    format!(
                        "vexmason log ({})\n",
                        Local::now().format("%a %d %b %Y, %I:%M%p")
                    )
                    .as_bytes(),
                )
                .await?;

            let config = match resolved_config_from_root(&root, &mut logs_file)
                .await
                .with_context(|| "couldn't resolve config")
            {
                Ok(config) => config,
                Err(e) => {
                    logs_file.write(format!("{e:?}\n").as_bytes()).await?;
                    eprintln!("{}", e.to_string());
                    std::process::exit(2);
                }
            };
            logs_file
                .write(format!("resolved config\n{:?}\n", config).as_bytes())
                .await?;

            match modify_args(
                &mut args,
                &ModifyOptions {
                    name: &config.name,
                    description: &config.description,
                    compile_target_file: &config.build_output(),
                    compile_minify: config.minify,
                    compile_defines: &config.defines,
                    app_data_location: &user_directory.join("AppData").join("Roaming"),
                },
                &mut logs_file,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    // make vex ignore the error but still visible to user
                    logs_file.write(format!("{e:?}\n").as_bytes()).await?;
                }
            }

            logs_file
                .write(
                    format!(
                        "Running:\n$ vexcom.old {}\n",
                        args.iter()
                            .map(|arg| {
                                if arg.contains(' ') {
                                    format!("\"{arg}\"")
                                } else {
                                    arg.to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                    .as_bytes(),
                )
                .await?;
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
                logs_file.try_clone().await.with_context(|| {
                    "failed to clone logs file to allow vexcom.old to write stderr to logs"
                })?,
                stderr(),
            ));

            let exit_status = child
                .wait()
                .await
                .with_context(|| "failed to wait on child")?;

            child_stderr_handle.await??;
            if exit_status.success() {
                logs_file.write("completed successfully".as_bytes()).await?;
                Ok(())
            } else {
                logs_file
                    .write("compiled, but vexcom.old exited with a non-zero exit code".as_bytes())
                    .await?;
                std::process::exit(exit_status.code().unwrap_or(1));
            }
        }
    }
}
