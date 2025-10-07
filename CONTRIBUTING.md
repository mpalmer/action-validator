# Overview

* If you have found a discrepancy in documented and observed behaviour, that
  is a bug. Feel free to [report it as an
  issue](https://github.com/mpalmer/action-validator/issues), providing
  sufficient detail to reproduce the problem.

* If you would like to add new behaviour, please submit a well-tested and
  well-documented [pull
  request](https://github.com/mpalmer/action-validator/pulls).

* At all times, abide by the Code of Conduct (CODE_OF_CONDUCT.md).

---

# Environment Setup

## Install Rust
Firstly, you'll need a Rust toolchain to make any changes to the core functionality of this project. We recommend [using `rustup`](https://www.rust-lang.org/tools/install), because that's what the Rust core team recommend.

To confirm that rust is installed, run the `cargo` command. If you don't receive the help docs output, you may need to add rust to your shell rc file.

## Git Submodule Setup
This repository uses [git submodules](https://git-scm.com/book/en/v2/Git-Tools-Submodules). Specifically for the use of [schemastore](https://github.com/SchemaStore/schemastore).

To setup the git submodule after cloning this repo to your local, you'll want to run the following commands:
1. `git submodule init`
2. `git submodule update`

It should look similar to the output below.

```bash
❯ git submodule init
Submodule 'src/schemastore' (https://github.com/SchemaStore/schemastore) registered for path 'src
/schemastore'
❯ git submodule update
Cloning into '/Users/someuser/action-validator/src/schemastore'...
Submodule path 'src/schemastore': checked out 'd3e6ab7727380b214acbab05570fb09a3e5d2dfc'
```

At this point, you should be all set to `cargo run`!

## Node/WASM Setup
If you plan to work on the WASM/Node bindings, you'll also need to install Node. We recommend using [NVM](https://github.com/nvm-sh/nvm) to install the Node version listed in `.nvmrc`.

Once Node is installed, run `npm install` at the root of the repository.

You should now be all set to run `npm build`, to build the Node/WASM bindings. Once built, run `npx action-validator` to run the CLI via the Node/WASM bindings.

# Running the Validator Locally

## `cargo run [FILE] -- [OPTIONS]`
`cargo run` will create a _debug_ executable and run the project. If this is your first time running the command, cargo will compile the development binary with `cargo build`. This will install all of the dependencies and create the debug binary `action-validator` in the `/target/debug/` directory. `cargo run` will then invoke this binary after creation.

One caveat if you're running with `cargo run`: if you want to supply the program with options, you need to use the `--` operator between `cargo run` and your provided options. This let's cargo know which flags are meant for cargo, and which are meant for the executable.

## `cargo build` && `./target/debug/action-validator [OPTIONS]`
As discussed in the prior section, `cargo build` install dependencies (if they're not cached) and build the development binary. This binary can then be invoked directly by running `./target/debug/action-validator`. This does **not** require the use of the `--` operator in between the binary and any provided options.

## Try It Yourself!

Run the command `cargo run -- --help`. You should see an output similar to the following.
```bash
❯ cargo run -- --help
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/action-validator --help`
A validator for GitHub Action and Workflow YAML files

Usage: action-validator [OPTIONS] [path_to_action_yaml]...

Arguments:
  [path_to_action_yaml]...  Input file

Options:
  -v, --verbose  Be more verbose
  -h, --help     Print help information
  -V, --version  Print version information
```

# Writing Tests

All tests live in the `tests` directory. Currently, this project implements snapshot testing,
but that's not to say you couldn't write unit or integration tests with the current structure.
To run the tests, simply run `cargo test` from the root directory. If you want to test a specific
feature, you can add the `-F {feature}` flag (e.g. `cargo test -F remote-checks`).

## Unit/Integration Tests
As of this writing, there are no unit or integration tests. If you are looking to write some, please
follow the directions in [this guide](https://doc.rust-lang.org/book/ch11-01-writing-tests.html).

## Snapshot Tests
A snapshot test is performed when we execute the cli and capture `stdout`, `stderr`, and/or an exit code.
When the tests are run, the results of the test must exactly match those of the previous run. For this project,
the snapshot tests are named in the format `{next_id}_{whats_being_tested}` (e.g. `011_remote_checks_failure`).

If you have made changes which will change the output of the program and cause snapshots to fail, you can run
`cargo test -F test-save-snapshots`. This feature causes the executed command to save the `stdout`, `stderr`, and/or
exit code to the specified testing directory.

If you are writing a net new test, you will need to create the test directory with your workflow or action file, and a
`test.json` file. Once you're done, you can save the results to that directy by running
`cargo test -F test-save-snapshots`.

The `test.json` file contains the test configuration. It can usually be left empty (i.e. `{}`).

```jsonc
// Example test.json
{
  "targets": {
    "node": false,
    "native": true
  },
  "cli_args": [
    "--rootdir",
    "tests/fixtures/011_subdir_globs/subdir",
    "tests/fixtures/011_subdir_globs/subdir/glob.yml"
  ]
}
```

# Testing Node/WASM Bindings

To test against the Node/WASM bindings, you can run `npm test`, or `npm test:dev` (to skip optimisations).
Note that Node support is considered experimental, and does not have one to one feature parity with the native binary yet.
As such, some tests may fail, even on `main`.
