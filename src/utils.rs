use crate::variable::Variable;
use std::env;
use std::os::unix::fs::symlink;
use std::process::Command;

pub fn get_config_dir(subdir: &String) -> String {
    let mut config_dir;
    match env::var("XDG_CONFIG_HOME") {
        Ok(dir) => {
            config_dir = dir
        }
        Err(_) => match env::var("HOME") {
            Ok(dir) => {
                config_dir = dir;
                config_dir.push_str(&"/.config".to_string());
            }
            Err(_) => {
                config_dir = ".".to_string();
            }
        },
    }
    config_dir.push_str(&"/".to_string());
    config_dir.push_str(subdir);
    return config_dir;
}

pub fn cmd(command: &String) -> Result<(), String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    return Ok(());
}

pub fn parse_file(filename: &String, variables: Vec<Variable>) -> Result<String, String> {
    let orig = std::fs::read_to_string(filename).expect("Failed to read file");
    let mut new = orig.clone();
    for variable in variables {
        let pattern = String::from("%{{") + &variable.name + "}}";
        new = new.replace(pattern.as_str(), &variable.value);
    }
    return Ok(new);
}

pub fn write_file(filename: &String, content: &String) -> Result<(), String> {
    std::fs::write(filename, content).expect("Failed to write file");
    return Ok(());
}

pub fn link_file(source: &String, target: &String) -> Result<(), String> {
    symlink(source, target).expect("Failed to link file");
    return Ok(());
}
