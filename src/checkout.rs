use std::process::Command;

use anyhow::Context;

use crate::util::CommandExt;

#[derive(Debug, Clone)]
pub struct GitRef {
    pub user: String,
    pub branch: String,
}

pub fn exec(gr: GitRef) -> anyhow::Result<()> {
    println!("Fetching remote...");

    Command::new("git")
        .arg("fetch")
        .arg(format!("https://github.com/{}/rust-clippy", gr.user))
        .arg(gr.branch)
        .run_success()
        .context("git fetch")?;

    println!("Checking out branch...");

    Command::new("git")
        .arg("checkout")
        .arg("FETCH_HEAD")
        .run_success()
        .context("git checkout")?;

    Ok(())
}
