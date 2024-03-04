use std::{
    env,
    path::{Path, PathBuf},
    process::{ExitCode, Stdio},
};

use anyhow::Context;

use flexi_logger::{FileSpec, LogSpecification, Logger};
use log::{debug, error, info};
use tokio::{fs, io::stderr, process::Command};
use vexmason::{
    compile_file,
    config::{self, resolved_config_from_root, root, CONFIG_FILE, CONFIG_OVERRIDES_FILE},
    installation_location::{self, VEXCOM_OLD_NAME},
    modify_args::{entry_point, has_write, modify_args, ModifyOptions},
    save_readable::save_readable,
};

#[tokio::main]
async fn main() -> ExitCode {
    match runtime().await {
        Ok(code) => code,
        Err(err) => {
            let mut err_str = format!("{}", err);
            if let Some(stripped) = err_str.strip_prefix("Error: ") {
                err_str = stripped.to_string();
            }
            eprintln!("{err_str}");
            ExitCode::FAILURE
        }
    }
}

async fn runtime() -> Result<ExitCode, anyhow::Error> {
    let mut args = env::args().skip(1);
    let vexcom_location = Into::<PathBuf>::into(
        args.next()
            .ok_or(anyhow::anyhow!("can't read vexcom location"))?,
    );
    let user_directory = installation_location::get_user_directory(Some(&vexcom_location))?;

    let args: Vec<String> = args.collect();

    match entry_point(&args) {
        None => {
            // if not in a vexmason project, just proxy to vexcom
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
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::from(
                    u8::try_from(
                        exit_status
                            .code()
                            .unwrap_or(1)
                            .clamp(u8::MIN.into(), u8::MAX.into()),
                    )
                    .unwrap(),
                ))
            }
        }
        Some(entry_point) => {
            let root = root(entry_point).with_context(
                || anyhow::anyhow!(
                    "failed to find your project! Make sure it has a vex_project_settings.json and a {} present inside the .vscode directory.",
                    config::CONFIG_FILE
                )
            )?;
            let build_dir = root.join("build");
            if !build_dir.exists() {
                fs::create_dir(root.join("build"))
                    .await
                    .with_context(|| "failed create the build output directory")?;
            }

            let logger = Logger::with(LogSpecification::trace())
                .adaptive_format_for_stderr(flexi_logger::AdaptiveFormat::Default)
                .log_to_file(
                    FileSpec::default()
                        .directory(build_dir)
                        .basename("vexmason")
                        .suppress_timestamp(),
                )
                .format_for_files(|w, now, record| {
                    write!(
                        w,
                        "[{}] {:<5} [{}] {}",
                        now.format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        record.module_path().unwrap_or("<unnamed>"),
                        &record.args()
                    )
                });
            #[cfg(feature = "stderr_log")]
            {
                logger.log_to_stderr();
            }
            logger.start()?;
            info!("started log");

            debug!("vexcom location: {:?}", vexcom_location);

            match compiling_runtime(args, &root, &user_directory, &vexcom_location).await {
                Err(err) => {
                    error!("{:?}", err);
                    Err(err)
                }
                Ok(exit_code) => match exit_code {
                    Some(exit_code) => Ok(ExitCode::from(exit_code)),
                    None => Ok(ExitCode::SUCCESS),
                },
            }
        }
    }
}

async fn compiling_runtime(
    mut args: Vec<String>,
    root: &Path,
    user_directory: &Path,
    vexcom_location: &Path,
) -> anyhow::Result<Option<u8>> {
    debug!("vexmason command-line arguments => {:?}", args);

    let config = resolved_config_from_root(&root)
        .await
        .with_context(|| "couldn't resolve config")?;

    info!(
        "resolved config from {} and {}",
        CONFIG_FILE, CONFIG_OVERRIDES_FILE
    );
    info!("{:#?}", config);

    if has_write(&args) {
        compile_file::compile_file(&compile_file::CompileFileOptions {
            input: &config.entry_file,
            output: Some(&config.build_output()),
            minify: config.minify,
            defines: &config.defines,
            app_data_location: &user_directory.join("AppData").join("Roaming"),
        })
        .await
        .with_context(|| "failed to compile file")?;
    } else {
        info!("no --write argument supplied, so skipping compile step");
    }

    modify_args(
        &mut args,
        &ModifyOptions {
            name: &config.name,
            description: &config.description,
            write_output: &config.build_output(),
        },
    )?;

    info!("running vexcom.old");
    debug!("args => {:?}", args);

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

    let child_stderr_handle = tokio::task::spawn(save_readable(
        child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to secure child stderr"))?,
        stderr(),
    ));

    // wait for vexcom
    let child_exit_status = child
        .wait()
        .await
        .with_context(|| "failed to wait on child")?;

    let child_stderr = child_stderr_handle.await??;
    if child_exit_status.success() {
        info!("vexcom completed successfully");
        Ok(None)
    } else {
        error!("vexcom exited with a non-zero exit code");
        error!(
            "vexcom stderr:\n{}",
            std::str::from_utf8(&child_stderr)
                .unwrap_or_else(|_| "failed to decode vexcom stderr to print")
        );
        Ok(Some(
            child_exit_status
                .code()
                .unwrap_or(1)
                .clamp(u8::MIN.into(), u8::MAX.into())
                .try_into()
                .unwrap(),
        ))
    }
}
