#![cfg_attr(
all(not(debug_assertions), target_os = "windows", feature = "gui"),
windows_subsystem = "windows"
)]

mod command;
mod script;
mod utils;

use command::Command;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "gui")]
fn gui() -> Result<(), std::io::Error> {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .run(context)
        .expect("error while running tauri application");
    Ok(())
}

#[cfg(not(feature = "gui"))]
fn gui() -> Result<(), std::io::Error> {
    println!("No command provided, exiting.");
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
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
                    println!(
                        r"cargo-commander 2.0.10
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
    -p, --parallel       Forces all commands to run in parallel"
                    );
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
        return gui();
    }

    let command_name = command_args.remove(0);

    let mut commands_map: HashMap<String, (PathBuf, Command)> =
        if commander_args.contains_key("file") {
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
            if command_name.starts_with("https://") || command_name.starts_with("http://") {
                return script::execute(command_name, "http", command_args);
            } else if std::path::Path::new(&command_name).is_file() {
                return script::execute(command_name, "file", command_args);
            } else {
                println!("Command not found!");
                return Ok(());
            }
        }
        Some((_, (dir, command))) => {
            let _ = std::env::set_current_dir(dir);
            for (k, v) in &command.args {
                command_args.retain(|x| *x != format!("{}={}", k, v));
            }
            let _ = command.execute(command_args);
        }
    }

    Ok(())
}
