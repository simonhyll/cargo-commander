use std::env;
use std::fs;
use toml::{Value, de::Error};
use std::process::exit;
use toml::value::Table;
use tokio::spawn;
use futures::FutureExt;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
struct Command {
    arguments: Vec<String>,
    shell: String,
    env: HashMap<String, String>,
    args: HashMap<String, String>,
    working_dir: String,
}

fn find_command(command: Vec<&str>, table: &Table) -> Option<Value> {
    let mut result: Option<Value> = Option::None;

    for (k, v) in table {
        if k == command[0] {
            match v {
                Value::String(_) => {
                    result = Option::Some(v.clone());
                    break;
                }
                Value::Integer(_) => {}
                Value::Float(_) => {}
                Value::Boolean(_) => {}
                Value::Datetime(_) => {}
                Value::Array(_) => {
                    result = Option::Some(v.clone());
                    break;
                }
                Value::Table(t) => {
                    if command.len() == 1 {
                        result = Option::Some(v.clone());
                        break;
                    } else if command.len() > 1 {
                        result = find_command(Vec::from(&command[1..]), t);
                    }
                }
            }
        }
    }

    result
}

#[derive(Debug)]
enum VecOrCommand {
    Vec(Vec<VecOrCommand>),
    Com(Command),
}

fn create_command_chain(value: Value, inherited: Option<HashMap<String, HashMap<String, String>>>) -> Vec<VecOrCommand> {
    let mut commands: Vec<VecOrCommand> = vec![];
    let mut parent_args: HashMap<String, String> = HashMap::new();
    let mut parent_env: HashMap<String, String> = HashMap::new();
    let mut parent_working_dir: String = "".to_string();
    if inherited.is_some() {
        let parent = inherited.unwrap();
        if parent.contains_key("args") {
            for (k, v) in parent.get("args").unwrap() {
                parent_args.insert(k.clone(), v.clone());
            }
        }
        if parent.contains_key("env") {
            for (k, v) in parent.get("env").unwrap() {
                parent_env.insert(k.clone(), v.clone());
            }
        }
        if parent.contains_key("working_dir") {
            parent_working_dir = parent.get("working_dir").unwrap().get("path").unwrap().to_string();
        }
    }
    match value {
        Value::String(s) => {
            commands.push(VecOrCommand::Com(Command {
                arguments: vec![s.clone()],
                shell: "".to_string(),
                env: parent_env.clone(),
                args: parent_args.clone(),
                working_dir: parent_working_dir.clone(),
            }))
        }
        Value::Integer(_) => {}
        Value::Float(_) => {}
        Value::Boolean(_) => {}
        Value::Datetime(_) => {}
        Value::Array(a) => {
            for x in a {
                let mut send_inherit: HashMap<String, HashMap<String, String>> = HashMap::new();
                send_inherit.insert("env".to_string(), parent_env.clone());
                send_inherit.insert("args".to_string(), parent_args.clone());
                let mut wd = HashMap::new();
                wd.insert("path".to_string(), parent_working_dir.clone());
                send_inherit.insert("working_dir".to_string(), wd.clone());
                for n in create_command_chain(x, Option::Some(send_inherit)) {
                    commands.push(n)
                }
            }
        }
        Value::Table(t) => {
            if t.contains_key("cmd") {
                let mut shell: String = "".to_string();
                if t.contains_key("shell") {
                    shell = t.get("shell").unwrap().to_string();
                }
                let mut working_dir: String = ".".to_string();
                if t.contains_key("working_dir") {
                    working_dir = t.get("working_dir").unwrap().to_string();
                }
                let mut parallel: bool = false;
                if t.contains_key("parallel") {
                    parallel = t.get("parallel").unwrap().as_bool().unwrap();
                }
                let mut env: HashMap<String, String> = HashMap::new();
                if t.contains_key("env") {
                    for x in t.get("env").unwrap().as_array().unwrap() {
                        match x {
                            Value::String(s) => {
                                let y: Vec<&str> = s.split("=").collect();
                                env.insert(y[0].to_string(), y[1].to_string());
                            }
                            Value::Integer(_) => {}
                            Value::Float(_) => {}
                            Value::Boolean(_) => {}
                            Value::Datetime(_) => {}
                            Value::Array(_) => {}
                            Value::Table(_) => {}
                        }
                    }
                }
                for (k, v) in parent_env.clone() {
                    env.insert(k, v);
                }
                let mut args: HashMap<String, String> = HashMap::new();
                if t.contains_key("args") {
                    for x in t.get("args").unwrap().as_array().unwrap() {
                        let values: Vec<&str> = x.as_str().unwrap().split("=").collect();
                        if values.len() == 2 {
                            args.insert(values[0].to_string(), values[1].to_string());
                        } else {
                            args.insert(values[0].to_string(), "".to_string());
                        }
                    }
                }
                for (k, v) in parent_args.clone() {
                    args.insert(k, v);
                }
                match t.get("cmd").unwrap() {
                    Value::String(s) => {
                        commands.push(VecOrCommand::Com(Command {
                            arguments: vec![s.clone()],
                            shell: shell,
                            env: env,
                            args: args,
                            working_dir: working_dir,
                        }))
                    }
                    Value::Integer(_) => {}
                    Value::Float(_) => {}
                    Value::Boolean(_) => {}
                    Value::Datetime(_) => {}
                    Value::Array(a) => {
                        let mut sub_commands: Vec<VecOrCommand> = vec![];
                        for x in a {
                            let mut send_inherit: HashMap<String, HashMap<String, String>> = HashMap::new();
                            let mut temp_args: HashMap<String, String> = HashMap::new();
                            for (k, v) in parent_args.clone() {
                                temp_args.insert(k, v);
                            }
                            for (k, v) in args.clone() {
                                temp_args.insert(k, v);
                            }
                            let mut temp_envs: HashMap<String, String> = HashMap::new();
                            for (k, v) in parent_env.clone() {
                                temp_envs.insert(k, v);
                            }
                            for (k, v) in env.clone() {
                                temp_envs.insert(k, v);
                            }
                            let mut temp_working_dir: HashMap<String, String> = HashMap::new();
                            temp_working_dir.insert("path".to_string(), working_dir.clone());
                            send_inherit.insert("args".to_string(), temp_args.clone());
                            send_inherit.insert("env".to_string(), temp_envs.clone());
                            send_inherit.insert("working_dir".to_string(), temp_working_dir.clone());
                            for n in create_command_chain(x.clone(), Option::Some(send_inherit)) {
                                if parallel {
                                    sub_commands.push(n)
                                } else {
                                    commands.push(n)
                                }
                            }
                        }
                        for n in sub_commands.iter_mut() {
                            match n {
                                VecOrCommand::Vec(_) => {}
                                VecOrCommand::Com(c) => {
                                    c.shell = shell.clone();
                                }
                            }
                        }
                        if parallel {
                            commands.push(VecOrCommand::Vec(sub_commands))
                        }
                        for n in commands.iter_mut() {
                            match n {
                                VecOrCommand::Vec(_) => {}
                                VecOrCommand::Com(c) => {
                                    c.shell = shell.clone()
                                }
                            }
                        }
                    }
                    Value::Table(_) => {}
                }
            } else {
                for (_, v) in t {
                    let mut send_inherit: HashMap<String, HashMap<String, String>> = HashMap::new();
                    send_inherit.insert("env".to_string(), parent_env.clone());
                    send_inherit.insert("args".to_string(), parent_args.clone());
                    for n in create_command_chain(v, Option::Some(send_inherit)) {
                        commands.push(n)
                    }
                }
            }
        }
    }
    commands
}

fn run_commands(command: VecOrCommand, mut path: PathBuf) -> BoxFuture<'static, ()> {
    async move {
        let mut handlers = vec![];
        match command {
            VecOrCommand::Vec(v) => {
                for c in v {
                    handlers.push(spawn(run_commands(c, path.clone())))
                }
                futures::future::join_all(handlers).await;
            }
            VecOrCommand::Com(mut c) => {
                let program: &str;
                c.shell = c.shell.strip_prefix("\"").unwrap_or_else(|| &c.shell).strip_suffix("\"").unwrap_or_else(|| &c.shell).to_string();
                c.working_dir = c.working_dir.strip_prefix("\"").unwrap_or_else(|| &c.working_dir).strip_suffix("\"").unwrap_or_else(|| &c.working_dir).to_string();
                let first: String = c.working_dir.chars().into_iter().take(1).collect();
                if first == ".".to_string() {
                    path.clear();
                }
                path.push(c.working_dir);
                let mut shell: Vec<&str> = c.shell.split(" ").collect();
                if c.shell == "".to_string() {
                    if cfg!(target_os = "windows") {
                        shell = Vec::from(["cmd", "/C"])
                    } else {
                        shell = Vec::from(["sh", "-c"])
                    }
                }
                program = shell[0];
                shell.remove(0);

                let args: Vec<String> = env::args().collect();

                let mut arguments_index = 2;
                if args.len() >= 3 && args[1] == "cmd" {
                    arguments_index = 3;
                }
                if args.len() >= arguments_index {
                    let mut args_map: HashMap<String, String> = HashMap::new();
                    for i in arguments_index..args.len() {
                        let values: Vec<&str> = args[i].split("=").collect();
                        args_map.insert(values[0].to_string(), values[1].to_string());
                    }
                    for i in 0..c.arguments.len() {
                        let mut new_arg = c.arguments[i].clone();
                        for (k, v) in &c.args {
                            if args_map.contains_key(k) {
                                let key = format!("{}{}", "$", k);
                                new_arg = new_arg.replace(key.as_str(), args_map.get(k).unwrap().as_str())
                            } else {
                                let key = format!("{}{}", "$", k);
                                new_arg = new_arg.replace(key.as_str(), v.as_str())
                            }
                        }
                        c.arguments[i] = new_arg
                    }
                }
                for n in shell.iter() {
                    c.arguments.insert(0, n.to_string());
                }
                let child_process = std::process::Command::new(program)
                    .args(c.arguments)
                    .envs(c.env)
                    .current_dir(path)
                    .spawn()
                    .expect("failed to execute process");
                let _ = child_process.wait_with_output();
            }
        }
    }.boxed()
}

async fn execute_command(command: &Value, path: PathBuf) {
    let commands: Vec<VecOrCommand> = create_command_chain(command.clone(), Option::None);
    for n in commands {
        run_commands(n, path.clone()).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("No command was given.");
        exit(0)
    }
    if args.len() == 2 && args[1] == "cmd" {
        println!("No command was given.");
        exit(0)
    }
    let mut command_index = 1;
    if args.len() >= 3 && args[1] == "cmd" {
        command_index = 2;
    }
    let run_command: Vec<&str> = args[command_index].as_str().split(".").collect();
    let mut using_cargo: bool = false;

    let mut cargo_path = env::current_dir().unwrap();
    let mut previous: bool = true;
    while previous {
        cargo_path.push("Cargo.toml");
        if fs::metadata(&cargo_path).is_ok() {
            break;
        } else {
            cargo_path.pop();
            if !cargo_path.pop() {
                previous = false;
                cargo_path.push("Cargo.toml");
            }
        }
    }

    if fs::metadata(&cargo_path).is_ok() {
        let cargo_toml_file: String = fs::read_to_string(&cargo_path).expect("Something went wrong reading the file");
        let cargo_toml: Value = toml::from_str(&cargo_toml_file)?;
        if cargo_toml.as_table().unwrap().contains_key("commands") {
            let command = find_command(run_command.clone(), cargo_toml.as_table().unwrap().get("commands").unwrap().as_table().unwrap());
            if command.is_some() {
                using_cargo = true;
                cargo_path.pop();
                execute_command(&command.unwrap(), cargo_path).await
            }
        }
    }
    let mut using_commands: bool = false;
    if !using_cargo {
        let mut commands_path = env::current_dir().unwrap();
        let mut previous: bool = true;
        while previous {
            commands_path.push("Commands.toml");
            if fs::metadata(&commands_path).is_ok() {
                break;
            } else {
                commands_path.pop();
                if !commands_path.pop() {
                    previous = false;
                    commands_path.push("Commands.toml");
                }
            }
        }
        if fs::metadata(&commands_path).is_ok() {
            let commands_toml_file: String = fs::read_to_string(&commands_path).expect("Something went wrong reading the file");
            let commands_toml: Value = toml::from_str(&commands_toml_file)?;
            let command = find_command(run_command.clone(), commands_toml.as_table().unwrap());
            if command.is_some() {
                using_commands = true;
                commands_path.pop();
                execute_command(&command.unwrap(), commands_path).await
            }
        }
    }
    if !using_cargo && !using_commands {
        println!("No command found!")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_tests() {
        let output = std::process::Command::new("cargo")
            .args(&["run", "tests"])
            .output()
            .expect("failed to execute process");
        assert_eq!(output.status.code().unwrap(), 0);
    }
    #[test]
    fn one() {
        let output = std::process::Command::new("cargo")
            .args(&["run", "tests.one"])
            .output()
            .expect("failed to execute process");
        let mut out = String::from_utf8(output.stdout).unwrap().clone();
        out = out.trim().to_string();
        out = out.replace("\r\n", " ");
        out = out.replace("\n", " ");
        let expected: String = "hello".to_string();
        assert_eq!(out, expected);
        assert_eq!(output.status.code().unwrap(), 0);
    }
    #[test]
    fn two() {
        let output = std::process::Command::new("cargo")
            .args(&["run", "tests.two"])
            .output()
            .expect("failed to execute process");
        let mut out = String::from_utf8(output.stdout).unwrap().clone();
        out = out.trim().to_string();
        out = out.replace("\r\n", " ");
        out = out.replace("\n", " ");
        let expected: String = "hello again".to_string();
        assert_eq!(out, expected);
        assert_eq!(output.status.code().unwrap(), 0);
    }
    #[test]
    fn seven() {
        let output = std::process::Command::new("cargo")
            .args(&["run", "tests.seven"])
            .output()
            .expect("failed to execute process");
        let mut out = String::from_utf8(output.stdout).unwrap().clone();
        out = out.trim().to_string();
        out = out.replace("\r\n", " ");
        out = out.replace("\n", " ");
        let expected: String = "Hello from python!".to_string();
        assert_eq!(out, expected);
        assert_eq!(output.status.code().unwrap(), 0);
    }
    #[test]
    fn eight() {
        let output = std::process::Command::new("cargo")
            .args(&["run", "tests.eight"])
            .output()
            .expect("failed to execute process");
        let mut out = String::from_utf8(output.stdout).unwrap().clone();
        out = out.trim().to_string();
        out = out.replace("\r\n", " ");
        out = out.replace("\n", " ");
        let expected: String = "Hello".to_string();
        assert_eq!(out, expected);
        assert_eq!(output.status.code().unwrap(), 0);
    }
    #[test]
    fn nine() {
        let output = std::process::Command::new("cargo")
            .args(&["run", "tests.nine"])
            .output()
            .expect("failed to execute process");
        let mut out = String::from_utf8(output.stdout).unwrap().clone();
        out = out.trim().to_string();
        out = out.replace("\r\n", " ");
        out = out.replace("\n", " ");
        let expected: String = "Hello".to_string();
        assert_eq!(out, expected);
        assert_eq!(output.status.code().unwrap(), 0);
    }
    #[test]
    fn ten() {
        let output = std::process::Command::new("cargo")
            .args(&["run", "tests.ten", "ARG1=hello"])
            .output()
            .expect("failed to execute process");
        let mut out = String::from_utf8(output.stdout).unwrap().clone();
        out = out.trim().to_string();
        out = out.replace("\r\n", " ");
        out = out.replace("\n", " ");
        let expected: String = "hello Default value".to_string();
        assert_eq!(out, expected);
        assert_eq!(output.status.code().unwrap(), 0);
    }
}
