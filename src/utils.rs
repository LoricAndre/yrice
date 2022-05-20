use std::env;
use std::process::Command;

pub fn get_config_dir(subdir: &String) -> String {
    let mut config_dir;
    match env::var("XDG_CONFIG_HOME") {
        Ok(dir) => config_dir = dir,
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

pub fn read_file_or_url(uri: &String) -> Result<String, String> {
    if std::fs::metadata(&uri).is_ok() { // It is a file
        return Ok(std::fs::read_to_string(uri).expect("Failed to read config file"));
    } else { // Try url
        return Ok(reqwest::blocking::get(uri).unwrap()
            .text()
            .expect("Failed to get config from uri"));
    }
}
