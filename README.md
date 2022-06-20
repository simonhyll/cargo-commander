<div align="center">

# Cargo Commander

The simple way of running commands

[![Crates.io](https://img.shields.io/crates/v/cargo-commander)](https://crates.io/crates/cargo-commander)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/adaptive-simon)](https://www.patreon.com/adaptive-simon)
[![GitHub last commit (branch)](https://img.shields.io/github/last-commit/adaptive-simon/cargo-commander/main)](https://github.com/adaptive-simon/cargo-commander/commit/main)
[![Build and test](https://github.com/adaptive-simon/cargo-commander/actions/workflows/build.yml/badge.svg)](https://github.com/adaptive-simon/cargo-commander/actions/workflows/build.yml)
[![Website](https://img.shields.io/website?down_color=red&down_message=offline&up_color=green&up_message=online&url=https%3A%2F%2Fadaptive-simon.github.io%2Fcargo-commander%2F)](https://adaptive-simon.github.io/cargo-commander/)
</div>

## Introduction

Cargo Commander serves to fill the gap in the `cargo` commands capabilities, namely not being able to run commands in a
similar fashion the way `npm` does with scripts. But while I was at it I decided to add some extra functionality to it.

New: In addition to running commands specified in either `Commands.toml`, `Cargo.toml` or `package.json`, functionality to execute scripts similar to how `cargo-script` does is being worked on. You can try it by either running a local script, `cargo cmd script.rs`, or running a remote script, `cargo cmd https://url.to.script`. This is currently in the early beta stages and functions by running `rustc input -o output`, then executing the output, so it's currently limited to using the standard library and the script has to be contained within that singular file. More features to come!

## Getting started

Either create your commands under a `[commands]` or `[package.metadata.commands]` section in `Cargo.toml`, or create a
new `Commands.toml` file. They all use the same syntax. Cargo commander also parses the `scripts` section
inside `package.json` if it's found. Normally scripts inside package.json are only allowed to be strings, but Cargo
Commander parses `package.json` by converting from json to toml, meaning you can add all the same options in json as you
can in toml.

```bash
# Install cargo-commander
cargo install cargo-commander
# Run your command
cargo cmd COMMAND

# Output of 'cargo cmd --help'
cargo-commander 2.0.8
A powerful tool for managing project commands

USAGE:
    cargo cmd [OPTIONS] [COMMAND/URL/FILE] [<ARGUMENTS>...]

ARGS:
    COMMAND              Name of the command to run
    URL                  Downloads a script, compiles then runs it
    FILE                 Compiles a file then runs it
    <ARGUMENTS>...       Arguments to the command

OPTIONS:
    -h, --help           Print help information
    -f, --file PATH      Custom path to command file to parse
    -p, --parallel       Forces all commands to run in parallel
```

## Command

A command can either be a string or a command object using the below fields to customize its behavior.

```text
cmd = String or Array, where an array can either contain string commands or other command objects
parallel = true/false, only makes a difference if the command object contains an array, makes all commands run in parallel
shell = String, the syntax is simply "program arg arg arg"
env = Array, an array of strings in the format "VAR=SOMETHING"
args = Array, an array of strings in the format "ARG=Default value", if no default is given an empty string is used
working_dir = String, path to the directory to use as working directory either relative to the command file or the current directory
```

### cmd

This can be either a string, a command object or an array of command objects.

If `cmd` is a multiline string the contents of the command is saved to a temporary file that gets safely deleted after
the program finishes. The arguments are then used to replace content within the string, and the only argument sent to
the shell is the path to the temporary file. We can use this behavior together with the `shell` option to create a file
whose absolute path gets passed as an argument to whatever program you specify as a shell. See the examples for how this
might look.

```toml
command = "echo Basic usage"
command = ["echo As an array"]
command = { cmd = "echo Hello" }
command = { cmd = ["echo Hello", "echo World"] }
command = { cmd = [{ cmd = "echo And hello again" }] }
command = { cmd = { cmd = "echo Hello again" } }
```

### parallel

Boolean, defaults to false. If the `cmd` of the command object is an array, all sub commands will be run at the same
time.

```toml
command = { cmd = ["echo first", "echo second", "echo third"], parallel = true }
```

### working_dir

String. The path where the command is supposed to execute in.

```toml
command = { cmd = "ls", working_dir = "src" }
command = { cmd = "ls", working_dir = "path/to/folder" }
```

### args

Array of strings in the format `args=["arg","argument=Default"]`. If an argument is a string without a default value set
it'll simply be replaced with an empty string.

```toml
command = { cmd = "echo $name", args = ["name=World"] }
```

### env

Array of strings in the format `env=["variable=Value"]`. Sets environment variables in the command. This is similar to
how `args` works, but the difference is
that `env` changes environment variables. This option is generally speaking not super useful, you probably want to
use `load_dotenv` instead.

```toml
# Unix
command = { cmd = "echo $HELLO", env = ["HELLO=World"] }
# Windows
command = { cmd = "echo %HELLO%", env = ["HELLO=World"] }
```

### load_dotenv

Boolean, defaults to false. Allows you to load environment variables from a .env file. The .env file should be located
in the same folder as the file that contains the command being run. This option is unaffected by the `working_dir`
option.

```toml
# Create a .env file with the contents "HELLO=World"
# Unix
command = { cmd = "echo $HELLO", load_dotenv = true }
# Windows
command = { cmd = "echo %HELLO%", load_dotenv = true }
```

### until

Integer. Which status code counts as a successful run. Normally we don't check the status code of the command, but with
this option we can tell the command to keep repeating until it reaches a specific exit code. If you set this
to `until=0` it would mean that you keep running the command until you reach a status 0 exit code. With `until=404` it
would keep running until you reach code 404. If you want to avoid infinite looping you should set `max_repeat` as well.

```toml
command = { cmd = "echo Hello", until = 0 }
```

### repeat

Integer. Minimum number of times the command is meant to run. If you run this together with `until` you'll always be
running the command at least this number of times.

```toml
command = { cmd = "echo Hello", repeat = 2 }
```

### delay

Integer or float. Amount of time to sleep before running the command. If you use this together with any of the
repetition based options this delay will be added before every run of the command.

```toml
command = { cmd = "echo Hello", delay = 2 }
command = { cmd = "echo Hello", delay = 3.7 }
```

### max_repeat

Integer. Sets the maximum number of times the command is allowed to retry. This is mostly useful when running together
with `until`.

```toml
command = { cmd = "echo Hello", repeat = 5, max_repeat = 1 }
command = { cmd = "echo Hello", until = 0, max_repeat = 1000 }
```

## Examples

### Opening documentation

I have a tendency to create multiple `mdbook` books for documenting my projects. It's really neat, but it can be a bit
of a bother to open them all one by one. So what I do is put the command to open each document under a `docs` section,
then run the section rather than each individual page, using the `-p` flag to make the section run in parallel.

```toml
# Commands.toml
[docs]
crate_one = { cmd = "mdbook serve --open --port 9001", working_dir = "crates/one/docs" }
crate_two = { cmd = "mdbook serve --open --port 9002", working_dir = "crates/two/docs" }
crate_three = { cmd = "mdbook serve --open --port 9003", working_dir = "crates/three/docs" }
crate_four = { cmd = "mdbook serve --open --port 9004", working_dir = "crates/four/docs" }
```

Now we can open all documents using a single command!

```bash
cargo cmd -p docs
```

### Passing a custom argument

Let's say you want to get a running shell inside a Kubernetes pod where you don't know the pod name beforehand, probably
because the pod was created by e.g. a deployment or a cronjob. There is a `kubectl` command you know of that can get you
a running shell inside the pod, the problem is that the command is pretty long and annoying to write every time, and
copy pasting the command from somewhere else every time gets repetitive really fast.

```toml
# Commands.toml
shell = { cmd = "kubectl exec --stdin --tty $pod -- /bin/bash", args = ["pod"] }
```

Now we can always get a shell to our pod by simple running the below simplified syntax. Now instead of having to both
find the name of your pod and copy it into the longer `kubectl` command, you can now easily remember that you have
a `shell` command that takes the argument `pod`.

```bash
cargo cmd shell pod=my-pod-123-654
```

### Running a script

With a mix of the `shell` option and the behavior we've set for when a command is a multiline string we can achieve
running scripts written directly in your command.

```toml
# Using python -c
hello_py_c = { cmd = "print('Hello')", shell = "python -c" }
# Using python and multiline string and an argument
hello_py = { cmd = """import os
print("Hello")
print("$name")
""", args = ["name=World"], shell = "python" }
```

You can then run it as follows:

```bash
cargo cmd hello_py_c
Hello
# Or multiline
cargo cmd hello_py
Hello
World
# ... With argument
cargo cmd hello_py name=Commander
Hello
Commander
```

### Keep retrying until command succeeds

Sometimes you run programs or write scripts that can fail. It's ok, it happens to everyone. Maybe it's a networked
resource it's trying to reach, or maybe a file on your computer. No matter what the reason, the program will sometimes
exit with a successful code `0`, other times it exits with code `404` because the page it tried to reach wasn't found.

We can easily create a simple retry loop using `until`, combined with `delay` so that the program isn't ran too often,
and `max_repeat` so that we don't try forever.

```toml
command = { cmd = "python script.py", until = 0, delay = 3, max_repeat = 1000 }
```

Running that command makes it keep retrying with a 3 seconds delay between retries. It will retry until it gets a 0
status returned, or a maximum of 1000 times.

## Notes

### Environment variables don't persist

I've tried to get this to work as intended but for now I've kind of given up on this since it appears to be anywhere
between impossible and really, really annoying to get to work right. So each each command will have a "fresh" set of
environment variables, if one command changes environment variables another command won't pick up on those changes, they
are run in different shells. You can either use the `env` option, or you can run a script in every command that sets up
environment variables, or you can use `load_dotenv` to load variables from a `.env` file. I consider these options to be
sufficient, if you really want variables to persist across commands you'll have to make a pull request with your
changes, or wait until I feel like delving deeper into the issue.
