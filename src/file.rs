extern crate serde_yaml;
extern crate casual;
use regex::Regex;
use casual::confirm;
use serde_yaml::Value;
use std::fs::{create_dir_all, remove_dir_all, remove_file};

use crate::utils;
use crate::variable::Variable;

#[derive(Debug)]
pub struct File {
    source: String,
    target: String,
    parse: bool,
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
            parse: parse,
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
        create_dir_all(&parts[1])
    }

    // Remove target file.
    // If the command errors, user will be prompted to confirm
    // to try treating the file as a directory.
    fn rm(&self) -> Result<(), String> {
        match remove_file(&self.target) {
            Ok(_) => {Ok(())}
            Err(_) => {
                if !confirm(format!("Failed to remove file: {}. YDots will try to treat it as a directory and remove it. Continue?: ", &self.target)) {
                    return Err(format!("Aborted, please remove it manually."));
                }
                match remove_dir_all(&self.target) {
                    Ok(_) => {return Ok(());}
                    Err(e) => {
                        return Err(format!("Error while removing {}: {}", self.target, e));
                    }
                }
            }
        }
    }

    // Link or parse and copy file to the target path.
    pub fn link(&self, variables: Vec<Variable>) -> Result<(), String> {
        match self.mkdir() {
            Ok(_) => {
                self.rm()?; // Remove target file if it exists. This replaces the `--force` option of GNU `ln`.
                if self.parse {
                    match utils::parse_file(&self.source, variables) {
                        Ok(parse_file) => utils::write_file(&self.target, &parse_file),
                        Err(e) => Err(e)
                    }
                } else {
                    utils::link_file(&self.source, &self.target)
                }
            }
            Err(e) => Err(e.to_string())
        }
    }
}
