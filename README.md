# clippyd
Even more dev tools for [clippy](https://github.com/rust-lang/rust-clippy).
Note that this is really only interesting to people *working on* clippy, not users of clippy.

Clippy already has a `clippy_dev` binary package which includes useful subcommands for automating things (like all the boilerplate for creating new lints), however there are some niche things that I often find myself doing where it would be useful to have a single command for, so this is that.

### Installation
`cargo install --git https://github.com/y21/clippyd`.
All commands must be run from within the clippy directory (though this is also checked and you will get an error if you try to run it anywhere else).

### Features
- [`clippyd checkout user:branch`](#checkout)
- [`clippyd profile path/to/project`](#profile)
- [`clippyd command path/to/project dev|release`](#command)


### Checkout
This lets you switch to a branch of someone else's fork of clippy.
This is particularily useful for reviewing more involved PRs in your editor locally with an LSP instead of the web diff view.

The format `user:branch` was chosen on purpose because it allows you to directly copy it from the PR.

### Profile
This [profiles](https://en.wikipedia.org/wiki/Profiling_(computer_programming)) clippy as it is linting a given crate (by path).

This requires `perf` to be installed on the system.
It outputs a `perf.data` file that can be viewed by running `perf report`.

You can also provide additional arguments to pass through to `perf`, e.g. `clippyd profile clippy_lints -- -g --call-graph=dwarf`

<details>
<summary>Example usage of profiling clippy on itself (`clippy_lints`)</summary>

```sh
# cwd=rust-clippy
$ clippyd profile clippy_lints
<output shortened>
[ perf record: Woken up 16 times to write data ]
[ perf record: Captured and wrote 4.006 MB perf.data (104789 samples) ]

$ perf report
# Overhead  Command        Shared Object           Symbol
 9.71%      rustc          clippy-driver           [.] quine_mc_cluskey::essential_minterms
 3.89%      rustc          librustc_driver.so      [.] <rustc_middle::ty::context::CtxtInterners>::intern_ty
 3.84%      rustc          librustc_driver.so      [.] rustc_span::edit_distance::edit_distance
 3.75%      rustc          libc.so.6               [.] _int_malloc
```
</details>

### Command
Prints the finalized clippy-driver command for linting a specified crate. This has all the extern dependencies included in the command.

This can be used in combination with many other tools: `clippyd profile path` could be a shortcut for `perf record $(clippyd command path)`.
This is also useful for debugging crashes with no stacktrace, as you can directly run clippy-driver under gdb with this.
