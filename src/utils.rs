use crate::Command;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::path::PathBuf;

fn handle_toml_value(value: &toml::Value) -> Vec<(String, Command)> {
    let mut map: Vec<(String, Command)> = vec![];
    for (key, value) in value.as_table().unwrap() {
        if value.is_table() {
            if value.as_table().unwrap().contains_key("cmd") {
                map.push((key.clone(), Command::from(value)))
            } else {
                for (k, command) in handle_toml_value(&value) {
                    map.push((format!("{}.{}", key, k), command));
                }
                let mut parent_cmd = Command::builder().build();
                parent_cmd.is_section = true;
                for (_, command) in handle_toml_value(&value) {
                    if !command.is_section {
                        parent_cmd.children.push(command);
                    }
                }
                map.push((key.clone(), parent_cmd));
            }
        } else {
            map.push((key.clone(), Command::from(value)))
        }
    }

    map
}

fn handle_toml(file_path: PathBuf) -> Vec<(String, Command)> {
    let mut map: Vec<(String, Command)> = vec![];
    let commands_toml_file: String =
        std::fs::read_to_string(&file_path).expect("Something went wrong reading the file");
    let commands_toml: toml::Value = toml::from_str(&commands_toml_file).expect("");
    let is_cargo_toml = file_path.file_name().unwrap() == "Cargo.toml";
    for (key, value) in commands_toml.as_table().unwrap() {
        if is_cargo_toml {
            if commands_toml.get("commands").is_some() {
                for (key, value) in commands_toml.get("commands").unwrap().as_table().unwrap() {
                    if value.is_table() {
                        if value.as_table().unwrap().contains_key("cmd") {
                            map.push((key.clone(), Command::from(value)));
                        } else {
                            for (k, command) in handle_toml_value(&value) {
                                map.push((format!("{}.{}", key, k), command));
                            }
                            let mut parent_cmd = Command::builder().build();
                            parent_cmd.is_section = true;
                            for (_, command) in handle_toml_value(&value) {
                                if !command.is_section {
                                    parent_cmd.children.push(command)
                                }
                            }
                            map.push((key.clone(), parent_cmd));
                        }
                    } else {
                        map.push((key.clone(), Command::from(value)));
                    }
                }
            }
            if commands_toml.get("package").is_some() {
                let pkg = commands_toml.get("package").unwrap();
                if pkg.get("metadata").is_some() {
                    let meta = pkg.get("metadata").unwrap();
                    if meta.get("commands").is_some() {
                        let cmds = meta.get("commands").unwrap();
                        for (key, value) in cmds.as_table().unwrap() {
                            if value.is_table() {
                                if value.as_table().unwrap().contains_key("cmd") {
                                    map.push((key.clone(), Command::from(value)));
                                } else {
                                    for (k, command) in handle_toml_value(&value) {
                                        map.push((format!("{}.{}", key, k), command));
                                    }
                                    let mut parent_cmd = Command::builder().build();
                                    parent_cmd.is_section = true;
                                    for (_, command) in handle_toml_value(&value) {
                                        if !command.is_section {
                                            parent_cmd.children.push(command)
                                        }
                                    }
                                    map.push((key.clone(), parent_cmd));
                                }
                            } else {
                                map.push((key.clone(), Command::from(value)));
                            }
                        }
                    }
                }
            }
        } else {
            if value.is_table() {
                if value.as_table().unwrap().contains_key("cmd") {
                    map.push((key.clone(), Command::from(value)));
                } else {
                    for (k, command) in handle_toml_value(&value) {
                        map.push((format!("{}.{}", key, k), command));
                    }
                    let mut parent_cmd = Command::builder().build();
                    parent_cmd.is_section = true;
                    for (_, command) in handle_toml_value(&value) {
                        if !command.is_section {
                            parent_cmd.children.push(command)
                        }
                    }
                    map.push((key.clone(), parent_cmd));
                }
            } else {
                map.push((key.clone(), Command::from(value)));
            }
        }
    }
    map
}

fn handle_json(file_path: PathBuf) -> Vec<(String, Command)> {
    let mut map: Vec<(String, Command)> = vec![];
    let json: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(file_path).unwrap()).unwrap();
    let scripts = json.get("scripts");
    if scripts.is_some() {
        for (k, v) in scripts.unwrap().as_object().unwrap() {
            let x = Command::from(v);
            map.push((k.clone(), x))
        }
    }
    map
}

pub fn get_commands_map(extra_file: Option<&String>) -> HashMap<String, (PathBuf, Command)> {
    let current_dir = std::env::current_dir().unwrap();
    let mut processing_dir = PathBuf::new();
    let mut files_to_read: Vec<PathBuf> = Vec::new();
    let skip_first = if cfg!(target_os = "windows") { 1 } else { 0 };
    for n in current_dir.iter().skip(skip_first) {
        processing_dir.push(n);
        let mut try_file = PathBuf::new();
        try_file.push(processing_dir.clone());
        // package.json
        try_file.push("package.json");
        if try_file.is_file() {
            files_to_read.push(try_file.clone());
        }
        try_file.pop();
        // Cargo.toml
        try_file.push("Cargo.toml");
        if try_file.is_file() {
            files_to_read.push(try_file.clone());
        }
        try_file.pop();
        // commands.json
        try_file.push("commands.json");
        if try_file.is_file() {
            files_to_read.push(try_file.clone());
        }
        try_file.pop();
        // Commands.toml
        try_file.push("Commands.toml");
        if try_file.is_file() {
            files_to_read.push(try_file.clone());
        }
        try_file.pop();
    }
    if extra_file.is_some() {
        let f = PathBuf::from(extra_file.unwrap());
        if f.is_file() {
            files_to_read.push(f);
        }
    }
    let mut map: HashMap<String, (PathBuf, Command)> = HashMap::new();

    files_to_read = files_to_read
        .iter()
        .filter(|x| x.is_file())
        .map(|x| x.to_owned())
        .collect();
    files_to_read.sort_unstable();
    files_to_read.dedup();
    // All files found! Quick workaround for sorting where toml is last
    let mut sorted_files: Vec<PathBuf> = Vec::new();
    for file_path in &files_to_read {
        if file_path.extension().unwrap() == "json" {
            sorted_files.push(file_path.clone());
        }
    }
    for file_path in &files_to_read {
        if file_path.extension().unwrap() == "toml" {
            sorted_files.push(file_path.clone());
        }
    }

    for file_path in sorted_files {
        let mut path = file_path.clone();
        path.pop();
        if file_path.extension().unwrap() == "toml" {
            for (name, command) in handle_toml(file_path) {
                if map.contains_key(&name) {
                    map.remove(&name);
                    map.insert(name, (path.clone(), command));
                } else {
                    map.insert(name, (path.clone(), command));
                }
            }
        } else if file_path.extension().unwrap() == "json" {
            for (name, command) in handle_json(file_path) {
                if map.contains_key(&name) {
                    map.remove(&name);
                    map.insert(name, (path.clone(), command));
                } else {
                    map.insert(name, (path.clone(), command));
                }
            }
        }
    }
    map
}

pub fn convert_json_to_toml(json: &serde_json::Value) -> toml::Value {
    let value: toml::Value;
    match json {
        serde_json::Value::Null => {
            value = toml::Value::from(false);
        }
        serde_json::Value::Bool(b) => {
            value = toml::Value::from(b.clone());
        }
        serde_json::Value::Number(n) => {
            value = toml::Value::from(n.as_f64().unwrap());
        }
        serde_json::Value::String(s) => {
            value = toml::Value::from(s.clone());
        }
        serde_json::Value::Array(a) => {
            let mut vec: Vec<toml::Value> = vec![];
            for n in a {
                vec.push(convert_json_to_toml(n))
            }
            value = toml::Value::from(vec);
        }
        serde_json::Value::Object(o) => {
            let mut map: toml::map::Map<String, toml::Value> = Default::default();
            for (k, v) in o {
                map.insert(k.clone(), convert_json_to_toml(v));
            }
            value = toml::Value::from(map);
        }
    }
    value
}

pub fn enable_all_parallel(map: &mut Vec<Command>) {
    for command in map {
        command.parallel = true;
        if command.children.len() > 0 {
            enable_all_parallel(command.children.borrow_mut());
        }
    }
}
