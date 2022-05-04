extern crate serde_yaml;
extern crate casual;
use regex::Regex;
use casual::confirm;
use serde_yaml::Value;
use std::fs;
use std::os::unix::fs::symlink;

use crate::variable::Variable;

#[derive(Debug)]
pub struct File {
    source: String,
    target: String,
    to_parse: bool,
}

impl File {
    pub fn new(yaml: &Value, source_dir: &String, target_dir: &String) -> File {
        let source_file = match yaml.as_str() {
            Some(s) => s.to_string(),
            None => yaml["source"].as_str().unwrap().to_string(),
        };
        let target_file = match yaml["target"].as_str() {
            Some(s) => s.to_string(),
            None => source_file.clone(),
        };
        let parse = match yaml["parse"].as_bool() {
            Some(b) => b,
            None => false,
        };
        let mut source_path = source_dir.clone();
        let mut target_path = target_dir.clone();
        if source_file != "" {
            source_path.push_str("/");
            source_path.push_str(&source_file);
            target_path.push_str("/");
            target_path.push_str(&target_file);
        }
        File {
            source: source_path,
            target: target_path,
            to_parse: parse,
        }
    }

    pub fn get_source(&self) -> &String {
        &self.source
    }

    // Create parent directory. This will fail most of the time.
    fn mkdir(&self) -> Result<(), std::io::Error> {
        let path = self.target.clone();
        let parts = Regex::new(r"^(.*)/([^/]*)$")
            .unwrap()
            .captures(&path)
            .unwrap();
        fs::create_dir_all(&parts[1])
    }

    // Remove target file.
    // If the command errors, user will be prompted to confirm
    // to try treating the file as a directory.
    fn rm(&self) -> Result<(), String> {
        match fs::remove_file(&self.target) {
            Ok(_) => {Ok(())}
            Err(_) => {
                if !confirm(format!("Failed to remove file: {}. YDots will try to treat it as a directory and remove it. Continue?: ", &self.target)) {
                    return Err(format!("Aborted, please remove it manually."));
                }
                match fs::remove_dir_all(&self.target) {
                    Ok(_) => {return Ok(());}
                    Err(e) => {
                        return Err(format!("Error while removing {}: {}", self.target, e));
                    }
                }
            }
        }
    }

    // Link or parse and copy file to the target path.
    pub fn copy(&self, variables: Vec<Variable>) -> Result<(), String> {
        match self.mkdir() {
            Ok(_) => {
                self.rm()?; // Remove target file if it exists. This replaces the `--force` option of GNU `ln`.
                if self.to_parse {
                    match self.parse(variables) {
                        Ok(parsed) => self.write(&parsed),
                        Err(e) => Err(e)
                    }
                } else {
                    self.link()
                }
            }
            Err(e) => Err(e.to_string())
        }
    }

    fn parse(&self, variables: Vec<Variable>) -> Result<String, String> {
        let orig = fs::read_to_string(&self.source)
            .expect("Failed to read file");
        let mut new = orig.clone();
        for variable in variables {
            let pattern = String::from("%{{") + &variable.name + "}}";
            new = new.replace(pattern.as_str(), &variable.value);
        }
        return Ok(new);
    }
    fn write(&self, content: &String) -> Result<(), String> {
        fs::write(&self.target, content)
            .expect("Failed to write file");
        return Ok(());
    }
    fn link(&self) -> Result<(), String> {
        symlink(&self.source, &self.target).expect("Failed to link file");
        return Ok(());
    }
}
