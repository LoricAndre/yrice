extern crate serde_yaml;
use log::{warn, debug};
use regex::Regex;
use serde_yaml::Value;
use std::fs::create_dir_all;

use crate::utils;
use crate::variable::Variable;

#[allow(dead_code)]
#[derive(Debug)]
pub struct File {
    pub source: String,
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

    fn mkdir(&self) -> Result<(), std::io::Error> {
        let path = self.target.clone();
        let parts = Regex::new(r"^(.*)/([^/]*)$")
            .unwrap()
            .captures(&path)
            .unwrap();
        create_dir_all(&parts[1])
    }

    fn rm(&self) {
        match std::fs::remove_file(&self.target) {
            Ok(_) => {}
            Err(_) => {
                match std::fs::remove_dir_all(&self.target) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Error while removing {}: {}", self.target, e)
                    },
                }
            }
        }
    }

    pub fn link(&self, variables: Vec<Variable>) -> Result<(), String> {
        match self.mkdir() {
            Ok(_) => {
                debug!("Copying {} to {}", self.source, self.target);
                self.rm();
                if self.parse {
                    match utils::parse_file(&self.source, variables) {
                        Ok(parse_file) => utils::write_file(&self.target, &parse_file),
                        Err(e) => Err(e),
                    }
                } else {
                    utils::link_file(&self.source, &self.target)
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
