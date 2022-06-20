use crate::utils::convert_json_to_toml;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;
use toml::Value;
//=============| STRUCT |==============//

#[derive(Debug)]
pub struct Command {
    // Whether this is a section or a singular command
    pub is_section: bool,
    // Minimum nr of times to repeat the command
    pub repeat: Option<i32>,
    // Maximum nr of times to repeat the command
    pub max_repeat: Option<i32>,
    // Keep repeating until it exits with this status code, or max_repeat is reached
    pub until: Option<i32>,
    // Environment variables to add to the command
    pub env: HashMap<String, String>,
    // The arguments to the command itself, not to std
    pub args: HashMap<String, String>,
    // Whether to load .env file
    pub load_dotenv: bool,
    // Which directory to run the command in
    pub working_dir: String,
    // The command to run
    pub command: Vec<String>,
    // Whether or not children should run in parallel
    pub parallel: bool,
    // Command to run before all other commands, e.g. to set up environment variables
    pub prefix: Vec<Command>,
    // Command to run after all other commands, e.g. for cleanup
    pub suffix: Vec<Command>,
    // How long to sleep before running
    pub delay: f64,
    // Which shell to run the command in
    pub shell: String,
    // File handles that need to stay open for the duration the command exists
    pub file_handles: Vec<NamedTempFile>,
    // Commands to run after the command finishes
    pub children: Vec<Command>,
}

#[derive(Debug)]
pub struct CommandBuilder {
    // Whether this is a section or a singular command
    pub is_section: bool,
    // Minimum nr of times to repeat the command
    pub repeat: Option<i32>,
    // Maximum nr of times to repeat the command
    pub max_repeat: Option<i32>,
    // Keep repeating until it exits with this status code, or max_repeat is reached
    pub until: Option<i32>,
    // Environment variables to add to the command
    pub env: HashMap<String, String>,
    // The arguments to the command itself, not to std
    pub args: HashMap<String, String>,
    // Whether to load .env file
    pub load_dotenv: bool,
    // Which directory to run the command in
    pub working_dir: String,
    // The command to run
    pub command: Vec<String>,
    // Whether or not children should run in parallel
    pub parallel: bool,
    // Command to run before all other commands, e.g. to set up environment variables
    pub prefix: Vec<Command>,
    // Command to run after all other commands, e.g. for cleanup
    pub suffix: Vec<Command>,
    // How long to sleep before running
    pub delay: f64,
    // Which shell to run the command in
    pub shell: String,
    // File handles that need to stay open for the duration the command exists
    pub file_handles: Vec<NamedTempFile>,
    // Commands to run after the command finishes
    pub children: Vec<Command>,
}

//=============| IMPL |==============//

impl Command {
    pub fn builder() -> CommandBuilder {
        CommandBuilder {
            is_section: false,
            repeat: None,
            max_repeat: None,
            until: None,
            env: HashMap::new(),
            load_dotenv: false,
            working_dir: "".to_string(),
            command: vec![],
            args: HashMap::new(),
            children: vec![],
            parallel: false,
            prefix: vec![],
            suffix: vec![],
            delay: 0.0,
            shell: "".to_string(),
            file_handles: vec![],
        }
    }
    pub fn execute(self, args: Vec<String>) -> Result<i32, std::io::Error> {
        let working_dir: String;
        if self.working_dir != "".to_string() {
            working_dir = self.working_dir.clone();
        } else {
            working_dir = ".".to_string();
        }

        let mut exit_status: i32;
        let mut repetitions: i32 = 0;
        let mut successes: i32 = 0;

        loop {
            if self.command.len() == 0 {
                exit_status = 0;
                break;
            }
            if self.delay > 0.0 {
                std::thread::sleep(std::time::Duration::from_secs_f64(self.delay));
            }
            repetitions += 1;

            let mut cmd = self.command.clone();
            let program = cmd.remove(0);

            let spawned_child;

            if cfg!(windows) {
                spawned_child = std::process::Command::new("cmd")
                    .arg("/C")
                    .arg(program)
                    .args(cmd)
                    .args(args.clone())
                    .envs(&self.env)
                    .current_dir(&working_dir)
                    .spawn()
                    .expect("failed to spawn");
            } else {
                spawned_child = std::process::Command::new(program)
                    .args(cmd)
                    .args(args.clone())
                    .envs(&self.env)
                    .current_dir(&working_dir)
                    .spawn()
                    .expect("failed to spawn");
            }
            let output = spawned_child.wait_with_output()?;
            exit_status = output.status.code().unwrap();
            // handle max_repeat
            if self.max_repeat.is_some() {
                if repetitions >= self.max_repeat.unwrap() {
                    break;
                }
            }
            // handle repeat
            if self.repeat.is_some() && self.until.is_none() {
                if repetitions < self.repeat.unwrap() {
                    continue;
                }
            }
            // handle until
            if self.until.is_some() {
                if exit_status != self.until.unwrap() {
                    continue;
                } else {
                    successes += 1;
                    if self.repeat.is_some() {
                        if successes < self.repeat.unwrap() {
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            // nothing set
            if self.repeat.is_none() {
                break;
            }
            if self.until.is_none() {
                break;
            }
        }

        if self.children.len() > 0 {
            if self.parallel {
                let mut handles = vec![];
                for child in self.children {
                    let cp = args.clone();
                    handles.push(std::thread::spawn(|| child.execute(cp)));
                }
                for h in handles {
                    // TODO: Handle status
                    let _ = h.join();
                }
            } else {
                for child in self.children {
                    // TODO: Handle status
                    let _ = child.execute(args.clone());
                }
            }
        }

        Ok(exit_status)
    }
}

impl CommandBuilder {
    pub fn build(self) -> Command {
        Command {
            is_section: self.is_section,
            repeat: self.repeat,
            max_repeat: self.max_repeat,
            until: self.until,
            env: self.env,
            load_dotenv: self.load_dotenv,
            working_dir: self.working_dir,
            command: self.command,
            args: self.args,
            children: self.children,
            parallel: self.parallel,
            prefix: self.prefix,
            suffix: self.suffix,
            delay: self.delay,
            shell: self.shell,
            file_handles: self.file_handles,
        }
    }
}

//=============| FROM |==============//

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        Command::from(&toml::Value::from(s))
    }
}

impl From<String> for Command {
    fn from(s: String) -> Self {
        Command::from(&toml::Value::from(s))
    }
}

impl From<&String> for Command {
    fn from(s: &String) -> Self {
        Command::from(&toml::Value::from(s.clone()))
    }
}

impl From<&toml::Value> for Command {
    fn from(v: &toml::Value) -> Self {
        let mut command = Command::builder();

        match v {
            toml::Value::String(s) => {
                let parts: Vec<String> = s.split(' ').map(|x| x.to_string()).collect();
                command.command = parts.clone()
            }
            toml::Value::Integer(_) => {}
            toml::Value::Float(_) => {}
            toml::Value::Boolean(_) => {}
            toml::Value::Datetime(_) => {}
            toml::Value::Array(_) => {}
            toml::Value::Table(_) => {}
        }

        if v.get("repeat").is_some() {
            let repeat = v.get("repeat").unwrap();
            match repeat {
                toml::Value::String(_) => {}
                toml::Value::Integer(i) => {
                    command.repeat = Some(i.clone() as i32);
                }
                toml::Value::Float(f) => {
                    command.repeat = Some(f.clone() as i32);
                }
                toml::Value::Boolean(_) => {}
                toml::Value::Datetime(_) => {}
                toml::Value::Array(_) => {}
                toml::Value::Table(_) => {}
            }
        }
        if v.get("max_repeat").is_some() {
            let max_repeat = v.get("max_repeat").unwrap();
            match max_repeat {
                toml::Value::String(_) => {}
                toml::Value::Integer(i) => command.max_repeat = Some(i.clone() as i32),
                toml::Value::Float(f) => command.max_repeat = Some(f.clone() as i32),
                toml::Value::Boolean(_) => {}
                toml::Value::Datetime(_) => {}
                toml::Value::Array(_) => {}
                toml::Value::Table(_) => {}
            }
        }
        if v.get("until").is_some() {
            let until = v.get("until").unwrap();
            match until {
                toml::Value::String(_) => {}
                toml::Value::Integer(i) => {
                    command.until = Some(i.clone() as i32);
                }
                toml::Value::Float(f) => {
                    command.until = Some(f.clone() as i32);
                }
                toml::Value::Boolean(_) => {}
                toml::Value::Datetime(_) => {}
                toml::Value::Array(_) => {}
                toml::Value::Table(_) => {}
            }
        }
        if v.get("env").is_some() {
            let env = v.get("env").unwrap();
            match env {
                toml::Value::String(_) => {}
                toml::Value::Integer(_) => {}
                toml::Value::Float(_) => {}
                toml::Value::Boolean(_) => {}
                toml::Value::Datetime(_) => {}
                toml::Value::Array(a) => {
                    for n in a {
                        if n.is_str() {
                            let x = n.as_str().unwrap();
                            let y: Vec<String> = x.split("=").map(|f| f.to_string()).collect();
                            if y.len() == 2 {
                                command.env.insert(y[0].clone(), y[1].clone());
                            }
                        }
                    }
                }
                toml::Value::Table(_) => {}
            }
        }
        if v.get("args").is_some() {
            let args = v.get("args").unwrap();
            match args {
                toml::Value::String(_) => {}
                toml::Value::Integer(_) => {}
                toml::Value::Float(_) => {}
                toml::Value::Boolean(_) => {}
                toml::Value::Datetime(_) => {}
                toml::Value::Array(a) => {
                    for n in a {
                        if n.is_str() {
                            let x = n.as_str().unwrap();
                            let y: Vec<String> = x.split("=").map(|f| f.to_string()).collect();
                            if y.len() == 1 {
                                command.args.insert(y[0].clone(), "".to_string());
                            } else if y.len() == 2 {
                                command.args.insert(y[0].clone(), y[1].clone());
                            }
                        }
                    }

                    let mut oargs: Vec<String> = std::env::args().collect();
                    if oargs.len() > 1 {
                        if oargs[0].contains("cargo-cmd") && oargs[1] == "cmd".to_string() {
                            // Removes possible cargo run part
                            oargs.remove(1);
                        }
                    }
                    // Removes the command path
                    oargs.remove(0);

                    let mut command_args: Vec<String> = vec![];
                    let mut all_found = false;
                    while oargs.len() > 0 {
                        if all_found {
                            command_args.push(oargs.remove(0));
                        } else {
                            if oargs[0].starts_with("-") {
                                if oargs[0] == "-f" || oargs[0] == "--file" {
                                    oargs.remove(0);
                                    oargs.remove(0);
                                } else {
                                    all_found = true;
                                }
                            } else {
                                all_found = true;
                            }
                        }
                    }
                    command_args.remove(0);
                    let oargs = command_args;

                    for a in oargs {
                        let v: Vec<String> = a.split("=").map(|x| x.to_string()).collect();
                        if v.len() == 2 {
                            // Ändra här
                            if command.args.get(&v[0]).is_some() {
                                let x = command.args.get_mut(&v[0]).unwrap();
                                *x = v[1].clone();
                            }
                        }
                    }
                }
                toml::Value::Table(_) => {}
            }
        }
        if v.get("load_dotenv").is_some() {
            let load_dotenv = v.get("load_dotenv").unwrap();
            match load_dotenv {
                Value::String(_) => {}
                Value::Integer(_) => {}
                Value::Float(_) => {}
                Value::Boolean(b) => {
                    command.load_dotenv = b.clone();
                    if *b {
                        let result: Vec<(String, String)> = dotenv::vars().collect();
                        for (k, v) in result {
                            if !command.env.contains_key(k.as_str()) {
                                command.env.insert(k, v);
                            }
                        }
                    }
                }
                Value::Datetime(_) => {}
                Value::Array(_) => {}
                Value::Table(_) => {}
            }
        }
        if v.get("working_dir").is_some() {
            let working_dir = v.get("working_dir").unwrap();
            match working_dir {
                toml::Value::String(s) => {
                    command.working_dir = s.clone();
                }
                toml::Value::Integer(_) => {}
                toml::Value::Float(_) => {}
                toml::Value::Boolean(_) => {}
                toml::Value::Datetime(_) => {}
                toml::Value::Array(_) => {}
                toml::Value::Table(_) => {}
            }
        }
        if v.get("parallel").is_some() {
            let parallel = v.get("parallel").unwrap();
            match parallel {
                toml::Value::String(_) => {}
                toml::Value::Integer(_) => {}
                toml::Value::Float(_) => {}
                toml::Value::Boolean(b) => {
                    command.parallel = b.clone();
                }
                toml::Value::Datetime(_) => {}
                toml::Value::Array(_) => {}
                toml::Value::Table(_) => {}
            }
        }
        if v.get("delay").is_some() {
            let delay = v.get("delay").unwrap();
            match delay {
                toml::Value::String(_) => {}
                toml::Value::Integer(i) => {
                    command.delay = i.clone() as f64;
                }
                toml::Value::Float(f) => {
                    command.delay = f.clone();
                }
                toml::Value::Boolean(_) => {}
                toml::Value::Datetime(_) => {}
                toml::Value::Array(_) => {}
                toml::Value::Table(_) => {}
            }
        }
        if v.get("prefix").is_some() {
            // Will only be added if I figure out how I can pass environment variables back and forth between commands
            todo!()
        }
        if v.get("suffix").is_some() {
            // We'll see if I add this
            todo!()
        }
        if v.get("shell").is_some() {
            let shell = v.get("shell").unwrap();
            match shell {
                Value::String(s) => {
                    command.shell = s.clone();
                }
                Value::Integer(_) => {}
                Value::Float(_) => {}
                Value::Boolean(_) => {}
                Value::Datetime(_) => {}
                Value::Array(_) => {}
                Value::Table(_) => {}
            }
        }

        if v.get("cmd").is_some() {
            match v.get("cmd").unwrap() {
                toml::Value::String(s) => {
                    if s.contains('\n') {
                        let mut cmd = s.clone();
                        for (arg, val) in &command.args {
                            cmd = cmd.replace(format!("${}", arg).as_str(), val.as_str())
                        }
                        let mut file =
                            tempfile::NamedTempFile::new().expect("failed to create temp file");
                        let _ = file.write(cmd.as_bytes()).expect("failed to write data");
                        let _ = file.flush().expect("failed to write data");
                        let path = file.path();
                        command.command = vec![path.to_str().unwrap().to_string()];
                        command.file_handles.push(file);
                    } else {
                        let parts: Vec<String> = s.split(' ').map(|x| x.to_string()).collect();
                        command.command = parts.clone();
                        for c in command.command.iter_mut() {
                            for (arg, val) in &command.args {
                                *c = c.replace(format!("${}", arg).as_str(), val.as_str())
                            }
                        }
                    }
                }
                toml::Value::Integer(_) => {}
                toml::Value::Float(_) => {}
                toml::Value::Boolean(_) => {}
                toml::Value::Datetime(_) => {}
                toml::Value::Array(a) => {
                    for n in a {
                        let mut cmd = Command::from(n);
                        if n.is_str() {
                            cmd.args = command.args.clone();
                            cmd.env = command.env.clone();
                            cmd.working_dir = command.working_dir.clone();
                        }
                        for c in cmd.command.iter_mut() {
                            if cmd.args.get(&c[1..c.len()]).is_some() {
                                *c = cmd.args.get(&c[1..c.len()]).unwrap().clone();
                            }
                        }
                        command.children.push(cmd);
                    }
                }
                toml::Value::Table(t) => {
                    if t.get("cmd").is_some() {
                        command.children.push(Command::from(t.get("cmd").unwrap()));
                    }
                }
            }
        }

        command.build()
    }
}

impl From<&serde_json::Value> for Command {
    fn from(v: &serde_json::Value) -> Self {
        Command::from(&convert_json_to_toml(v))
    }
}
