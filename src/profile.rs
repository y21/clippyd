use std::path::Path;
use std::process::Command;

use anyhow::Context;

use crate::command;
use crate::command::ClippyDriverCommandParts;
use crate::command::Profile;
use crate::util::CommandExt;

pub fn exec(path: &Path, perf: &[String]) -> anyhow::Result<()> {
    let ClippyDriverCommandParts {
        envs,
        mut exec_and_args,
        clippy_cwd,
        libdir,
    } = command::clippy_driver_command_for_crate(path, Profile::Release, true)?;

    exec_and_args.insert_str(
        0,
        &format!(
            "perf record -o {} {}",
            clippy_cwd.join("perf.data").to_str().unwrap(),
            perf.iter()
                .cloned()
                .intersperse(" ".into())
                .collect::<String>()
        ),
    );

    println!("Running perf");
    Command::new("sh")
        .arg("-c")
        .arg(envs + " " + &exec_and_args)
        .env("LD_LIBRARY_PATH", libdir)
        .current_dir(path)
        .run_success()
        .context("perf record")?;

    Ok(())
}
