#![feature(let_chains, iter_intersperse)]
#![warn(clippy::pedantic)]

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::ensure;
use anyhow::Context;
use checkout::GitRef;
use clap::Parser;

mod checkout;
mod profile;
mod util;

#[derive(Debug, Parser)]
enum Args {
    /// Check out a remote branch on someone else's fork of rust-clippy.
    Checkout {
        /// The git ref. This should contain the username and the branch, delimited by a colon, e.g. `y21:branch1`.
        ///
        /// Tip: when looking at a PR, you can directly copy the user:branch pair under the title
        #[arg(value_parser = |name: &str| {
            let (user, branch) = name.split_once(':').context("missing : in git ref")?;
            anyhow::Ok(GitRef { user: user.to_owned(), branch: branch.to_owned() })
        })]
        git_ref: GitRef,
    },
    /// Profile clippy on a particular crate. This will create a `perf.data` file in the current directory which can be inspected with `perf report`.
    ///
    /// This also expects `perf` to be installed.
    Profile {
        /// A path to the crate that clippy will be profiled on
        path: PathBuf,
        /// Arguments to pass through `perf record`
        perf_args: Vec<String>,
    },
}

/// Asserts that we're in the clippy repo.
fn assert_clippy_cwd() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let cargo_toml = fs::read_to_string(cwd.join("Cargo.toml"))?;

    ensure!(
        cargo_toml.contains("name = \"clippy\""),
        "missing clippy package name in manifest"
    );

    anyhow::Ok(())
}

fn main() -> anyhow::Result<()> {
    assert_clippy_cwd().context("failed clippy working directory check! make sure that you are running this from within the clippy repository")?;

    let args = Args::parse();
    match args {
        Args::Checkout { git_ref } => checkout::exec(git_ref),
        Args::Profile { path, perf_args } => profile::exec(&path, &perf_args),
    }
}
