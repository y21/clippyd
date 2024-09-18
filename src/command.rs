use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use anyhow::ensure;
use anyhow::Context;
use cargo_manifest::Manifest;
use cargo_manifest::Value;

use crate::util;
use crate::util::CommandExt;

fn ensure_has_debug(manifest: &Manifest) -> anyhow::Result<()> {
    let profile = manifest
        .profile
        .as_ref()
        .context("missing `profile` section")?;

    let release = profile
        .release
        .as_ref()
        .context("missing `profile.release` section")?;

    ensure!(
        release.debug == Some(Value::Boolean(true)),
        "missing `debug = true` in `profile.release`"
    );

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum Profile {
    Dev,
    Release,
}
pub struct ClippyDriverCommandParts {
    pub envs: String,
    pub exec_and_args: String,
    pub clippy_cwd: PathBuf,
    pub libdir: PathBuf,
}

pub fn clippy_driver_command_for_crate(
    path: &Path,
    profile: Profile,
    verbose: bool,
) -> anyhow::Result<ClippyDriverCommandParts> {
    macro_rules! println {
        ($($t:tt)*) => {
            if verbose {
                ::std::println!($($t)*)
            }
        };
    }

    println!("Building cargo-clippy and clippy-driver (this may take a while)...");
    Command::new("cargo")
        .args(match profile {
            Profile::Dev => &["b", "--bin", "cargo-clippy", "--bin", "clippy-driver"] as &[_],
            Profile::Release => &["b", "--bin", "cargo-clippy", "--bin", "clippy-driver", "-r"],
        })
        .run_success()?;

    let clippy_cwd = env::current_dir()?;
    let manifest =
        Manifest::from_path(clippy_cwd.join("Cargo.toml")).context("reading manifest")?;
    if matches!(profile, Profile::Release) && env::var("CLIPPYD_IGNORE_MANIFEST_CHECKS").is_err() {
        ensure_has_debug(&manifest)
            .context("rust-clippy/Cargo.toml checks failed; consider fixing them or set `CLIPPYD_IGNORE_MANIFEST_CHECKS` to skip them (results might be inaccurate)")?;
    }

    let clippy_target_cwd = clippy_cwd.join(match profile {
        Profile::Dev => "target/debug",
        Profile::Release => "target/release",
    });
    let cargo_clippy = clippy_target_cwd.join("cargo-clippy");

    let libdir = util::get_libdir()?;

    println!("Building crate dependencies and getting rustc command");
    let mut clippy_out = Command::new(cargo_clippy)
        .current_dir(path)
        .env("LD_LIBRARY_PATH", &libdir)
        .env("CARGO_INCREMENTAL", "0")
        .args(["--", "-vvv"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("cargo-clippy")?;

    let target_pkg_name = util::cargo_pkg_name(path)?;

    let stderr = BufReader::new(clippy_out.stderr.take().expect("stderr is piped"));
    let (envs, command) = 'cmd: {
        for line in stderr.lines() {
            let line = line?;
            let line = line.trim();

            if let Some(command) = line.strip_prefix("Running ")
                && let args = command.split(' ').collect::<Vec<_>>()
                && args.iter().any(|var| {
                    var.split_once('=').is_some_and(|(name, value)| {
                        name == "CARGO_PKG_NAME" && value == target_pkg_name
                    })
                })
                && let Some(executable_index) =
                    args.iter().position(|arg| arg.ends_with("clippy-driver"))
            {
                let (envs, cmd_args) = args.split_at(executable_index);
                let collect_into_string = |strs: &[&str]| {
                    strs.iter()
                        .fold(String::new(), |prev, cur| prev + " " + cur)
                };

                let mut envs = collect_into_string(envs);
                strip_in_place(&mut envs, " `");

                let mut cmd = collect_into_string(cmd_args);
                strip_in_place(&mut cmd, "`");

                let cmd = cmd.replace(
                    "--error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat",
                    "",
                );

                break 'cmd (envs, cmd);
            }
        }

        panic!("'Running' line was not present in cargo clippy -vvv output");
    };
    // No need to even bother finishing. We just needed the full command.
    clippy_out.kill().unwrap();
    clippy_out.wait().unwrap();

    Ok(ClippyDriverCommandParts {
        envs,
        exec_and_args: command,
        clippy_cwd,
        libdir,
    })
}

fn strip_in_place(s: &mut String, remove: &str) {
    if let Some(removed_prefix) = s.strip_prefix(remove) {
        s.drain(0..s.len() - removed_prefix.len());
    }

    if let Some(without_suffix) = s.strip_suffix(remove) {
        s.truncate(without_suffix.len());
    }
}

pub fn exec(crate_path: &Path, profile: Profile) -> anyhow::Result<()> {
    let ClippyDriverCommandParts {
        envs,
        exec_and_args,
        ..
    } = clippy_driver_command_for_crate(crate_path, profile, true)?;
    println!("{envs} {exec_and_args}");
    Ok(())
}
