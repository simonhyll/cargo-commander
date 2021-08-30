<div align="center">

# Cargo Commander
The simple way of running commands
</div>

## Introduction
Cargo Commander serves to fill the gap in the `cargo` commands capabilities, namely not being able to run
commands in a similar fashion that `npm` does with scripts.

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
Here are some examples of how you can set up commands.
```toml
# Cargo.toml
[commands]
hello = "echo world"
multiple = ["echo first", "echo second"]
advanced = { cmd = "print('Hello from python!')", shell = "python -c"}
[commands.test]
hello = "echo test"
```

```toml
# Commands.toml
hello = "echo world"
super_advanced = { cmd = [
    "echo we",
    "echo run",
    "echo in",
    "echo parallel"
], parallel = true}
[git]
status = "git status"
```

If  you want to see more examples, check out the `Commands.toml` file in the repository.

## Types of commands
First we have the simplest of the commands, a simple string. It will be executed in the system default
shell, either `cmd /C` or `sh -c` depending on if you're on Windows or not.

```toml
example = "echo Hello"
```

Secondly, you can give an array of strings. This will run each command in sequence.

```toml
example = [
    "echo First",
    "echo Second",
    { cmd = "echo Third" }
]
```

A command object has a very simple syntax to customize its run.
```toml
example = { cmd = ["echo One", "echo Two"], parallel = true}
```

Command object fields:

```
cmd = String or Array, where an array can either contain string commands or other command objects
parallel = true/false, only makes a difference if the command object contains an array, makes all commands run in parallel
shell = String, the syntax is simply "program arg arg arg"
env = Array, an array of strings in the format "VAR=SOMETHING"
```

You can structure your commands in sections, and you can run entire sections if you like.

For example:
```toml
[numbers]
first = "echo first"
second = "echo second"
third = "echo third"
[numbers.tens]
ten = "echo ten"
twenty = "echo twenty"
thirty = "echo thirty"
```
With the above configuration, running `cargo cmd numbers.tens` would run all commands in the `numbers.tens` section.

Running `cargo cmd numbers` would run all commands in the `numbers` section, INCLUDING the ones in `numbers.tens`.

Running `cargo cmd numbers.first` will only run the `first` command.
