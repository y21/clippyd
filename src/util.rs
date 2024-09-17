use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use anyhow::anyhow;
use anyhow::Context;
use cargo_manifest::Manifest;

pub trait CommandExt {
    fn run_success(&mut self) -> anyhow::Result<String>;
}

impl CommandExt for Command {
    fn run_success(&mut self) -> anyhow::Result<String> {
        let out = self.stderr(Stdio::inherit()).output().context("execute")?;
        if out.status.success() {
            Ok(String::from_utf8(out.stdout).unwrap())
        } else {
            Err(anyhow!("failed to execute command"))
        }
    }
}

pub fn get_libdir() -> anyhow::Result<PathBuf> {
    let mut libdir = Command::new("rustc")
        .args(["--print", "target-libdir"])
        .run_success()
        .context("getting the libdir")?;

    libdir.truncate(libdir.trim_end().len());
    Ok(PathBuf::from(libdir))
}

pub fn cargo_pkg_name(proj_path: &Path) -> anyhow::Result<String> {
    let manifest =
        Manifest::from_path(proj_path.join("Cargo.toml")).context("reading manifest file")?;
    Ok(manifest.package.context("missing `package` section")?.name)
}
