mod command;
mod utils;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::path::PathBuf;
use command::Command;

fn main() -> Result<(), ()> {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if args[0].contains("cargo-cmd") && args[1] == "cmd".to_string() {
            args.remove(1);
        }
    }
    args.remove(0);

    let mut command_args: Vec<String> = vec![];
    let mut commander_args: HashMap<String, String> = HashMap::new();

    let mut all_found = false;
    while args.len() > 0 {
        if all_found {
            command_args.push(args.remove(0));
        } else {
            if args[0].starts_with("-") {
                if args[0] == "-f" || args[0] == "--file" {
                    args.remove(0);
                    commander_args.insert("file".to_string(), args.remove(0));
                } else if args[0] == "-p" || args[0] == "--parallel" {
                    commander_args.insert("parallel".to_string(), args.remove(0));
                } else if args[0] == "-h" || args[0] == "--help" {
                    println!(r"cargo-commander 2.0.0
A powerful tool for managing project commands

USAGE:
    cargo cmd [OPTIONS] [COMMAND] [<ARGUMENTS>...]

ARGS:
    COMMAND           Name of the command to run
    <ARGUMENTS>...    Arguments to the command

OPTIONS:
    -h, --help           Print help information
    -f, --file PATH      Custom path to command file to parse
    -p, --parallel       Forces all commands to run in parallel");
                    return Ok(());
                } else {
                    all_found = true;
                }
            } else {
                all_found = true;
            }
        }
    }

    if command_args.len() == 0 {
        println!("No command provided, exiting.");
        return Ok(());
    }

    let command_name = command_args.remove(0);

    let mut commands_map: HashMap<String, (PathBuf, Command)> = if commander_args.contains_key("file") {
        utils::get_commands_map(commander_args.get("file"))
    } else {
        utils::get_commands_map(None)
    };

    if commander_args.contains_key("parallel") {
        for (_, (_, command)) in commands_map.iter_mut() {
            command.parallel = true;
            if command.children.len() > 0 {
                utils::enable_all_parallel(command.children.borrow_mut())
            }
        }
    }

    let cmd = commands_map.remove_entry(&command_name);
    match cmd {
        None => {
            println!("Command not found!");
            return Ok(());
        }
        Some((_, (dir, command))) => {
            let _ = std::env::set_current_dir(dir);
            let _ = command.execute();
        }
    }
    Ok(())
}
