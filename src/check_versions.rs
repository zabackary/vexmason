use std::error::Error;
use std::fmt;
use std::io::Read;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum VersionError {
    NotFound {
        prog: String,
    },
    ParseFailure,
    IoErr(std::io::Error),
    BadVersion {
        prog: String,
        version: semver::Version,
        version_requirement: semver::VersionReq,
    },
}

impl Error for VersionError {}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VersionError::NotFound { prog } => write!(
                f,
                "failed to find {prog}. check that it's installed and on your PATH."
            ),
            VersionError::ParseFailure => write!(f, "failed to parse version"),
            VersionError::IoErr(err) => err.fmt(f),
            VersionError::BadVersion {
                prog,
                version,
                version_requirement,
            } => write!(
                f,
                "{} {} doesn't satisfy required version {}",
                prog, version, version_requirement
            ),
        }
    }
}

pub fn check_versions() -> Result<(), VersionError> {
    check_version("python", "Python ", "^3.10")?;
    check_version("git", "git version ", "^2.40")?;
    Ok(())
}

fn check_version(prog: &str, stdout_prefix: &str, req: &str) -> Result<(), VersionError> {
    let version_requirement = semver::VersionReq::parse(req).unwrap();
    let version = semver::Version::parse(&version(prog, stdout_prefix)?)
        .map_err(|_| VersionError::ParseFailure)?;
    if version_requirement.matches(&version) {
        Ok(())
    } else {
        Err(VersionError::BadVersion {
            prog: prog.into(),
            version_requirement,
            version,
        })
    }
}

fn version(prog: &str, prefix: &str) -> Result<String, VersionError> {
    let cmd = Command::new(prog)
        .arg("--version")
        .stdout(Stdio::piped())
        .spawn();
    match cmd {
        Ok(mut child) => {
            child.wait().map_err(VersionError::IoErr)?;
            let mut stdout = String::new();
            child
                .stdout
                .take()
                .unwrap()
                .read_to_string(&mut stdout)
                .map_err(VersionError::IoErr)?;
            let version = stdout
                .strip_prefix(prefix)
                .ok_or(VersionError::ParseFailure)?
                .trim()
                .split(".")
                .take(3)
                .collect::<Vec<&str>>()
                .join(".");
            Ok(version)
        }
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                Err(VersionError::NotFound { prog: prog.into() })
            } else {
                Err(VersionError::IoErr(e))
            }
        }
    }
}
