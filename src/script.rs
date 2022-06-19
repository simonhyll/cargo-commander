fn compile_run_rust(path: &String, args: Vec<String>) -> Result<(), std::io::Error> {
    let tmp_dir = tempfile::Builder::new().tempdir()?;
    let fname = tmp_dir
        .path()
        .join("script.bin")
        .into_os_string()
        .into_string()
        .unwrap();
    let spawned_child = std::process::Command::new("rustc")
        .arg(path)
        .args(["-o", fname.as_str()])
        .spawn()
        .expect("failed to spawn");
    let output = spawned_child.wait_with_output()?;
    let exit_status = output.status.code().unwrap();

    if exit_status != 0 {
        panic!("failed to compile script");
    }

    let spawned_child = std::process::Command::new(fname.as_str())
        .args(args)
        .spawn()
        .expect("failed to spawn");
    let _output = spawned_child.wait_with_output()?;
    Ok(())
}

fn execute_http(command_name: String, args: Vec<String>) -> Result<(), std::io::Error> {
    let req = reqwest::blocking::get(&command_name);
    match req {
        Ok(response) => {
            let tmp_dir = tempfile::Builder::new().tempdir()?;
            let fname_str = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("tmp.bin");
            let fname = tmp_dir.path().join(fname_str);
            let mut dest = std::fs::File::create(&fname)?;
            let content = response.text().unwrap();
            std::io::copy(&mut content.as_bytes(), &mut dest)?;

            let path = &fname.into_os_string().into_string().unwrap();

            return compile_run_rust(&path, args);
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
    Ok(())
}

fn execute_file(command_name: String, args: Vec<String>) -> Result<(), std::io::Error> {
    return compile_run_rust(&command_name, args);
}

pub fn execute(
    command_name: String,
    variant: &str,
    args: Vec<String>,
) -> Result<(), std::io::Error> {
    if variant == "http" {
        return execute_http(command_name, args);
    } else if variant == "file" {
        return execute_file(command_name, args);
    } else {
        return Ok(());
    }
}
