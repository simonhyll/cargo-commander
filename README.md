<div align="center">

# Cargo Commander

The simple way of running commands

[![Crates.io](https://img.shields.io/crates/v/cargo-commander)](https://crates.io/crates/cargo-commander)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/seranth)](https://github.com/seranth/cargo-commander/blob/main/.github/FUNDING.yml)
[![GitHub last commit (branch)](https://img.shields.io/github/last-commit/seranth/cargo-commander/main)](https://github.com/seranth/cargo-commander/commit/main)
[![Build and test](https://github.com/seranth/cargo-commander/actions/workflows/build.yml/badge.svg)](https://github.com/seranth/cargo-commander/actions/workflows/build.yml)
[![Website](https://img.shields.io/website?down_message=offline&up_message=online&url=https%3A%2F%2Fseranth.github.io%2Fcargo-commander%2F)](https://seranth.github.io/cargo-commander/)
</div>

## Introduction

Cargo Commander serves to fill the gap in the `cargo` commands capabilities, namely not being able to run commands in a
similar fashion that `npm` does with scripts.

## Getting started

Either create your commands under a `[commands]` section in your `Cargo.toml` file, or create a new
`Commands.toml` file which uses the exact same syntax as if it had been under the commands section.

```shell
# Install cargo-commander
cargo install cargo-commander
# Run your command
cargo cmd COMMAND
```

## How to use

Read the full guide in our [wiki](https://github.com/seranth/cargo-commander/wiki).

A command can either be a string or a command object using the below fields to customize its behavior.

```text
cmd = String or Array, where an array can either contain string commands or other command objects
parallel = true/false, only makes a difference if the command object contains an array, makes all commands run in parallel
shell = String, the syntax is simply "program arg arg arg"
env = Array, an array of strings in the format "VAR=SOMETHING"
args = Array, an array of strings in the format "ARG=Default value", if no default is given an empty string is used
```

Here are some examples of how you can set up commands.

```toml
# Cargo.toml
[commands]
hello = "echo world"
multiple = ["echo first", "echo second"]
advanced = { cmd = "print('Hello from python!')", shell = "python -c" }
[commands.test]
hello = "echo test"
with_arguments = { cmd = "echo $ARG1 $ARG2", args = ["ARG1", "ARG2=Default value"] }
```

```toml
# Commands.toml
hello = "echo world"
super_advanced = { cmd = [
    "echo we",
    "echo run",
    "echo in",
    "echo parallel"
], parallel = true }
[git]
status = "git status"
```

If you want to see more examples, check out the `Commands.toml` file in the repository.
