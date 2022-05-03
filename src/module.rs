extern crate serde_yaml;
use serde_yaml::Value;
extern crate resolve_path;
use log::{error, info, warn};
use resolve_path::PathResolveExt;
use std::path::Path;

use crate::file::File;
use crate::utils;
use crate::variable::Variable;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Module {
    pub name: String,
    enabled: bool,
    source_dir: String,
    target_dir: String,
    package_name: Option<String>,
    dependencies: Vec<String>,
    files: Vec<File>,
    custom_steps: Vec<String>,
}

impl Module {
    pub fn new(name: &String, yaml: &Value, dots_dir: &String) -> Module {
        let dirname = match yaml["dirname"].as_str() {
            Some(dirname) => dirname,
            None => name.as_str(),
        };
        let source_dir = Path::new(dots_dir)
            .join(dirname)
            .to_str()
            .unwrap()
            .resolve()
            .to_str()
            .unwrap()
            .to_string();
        let enabled = match yaml["enabled"].as_bool() {
            Some(enabled) => enabled,
            None => true,
        };
        let target_dir = match yaml["targetDir"].as_str() {
            Some(target_dir) => target_dir.resolve().to_str().unwrap().to_string(),
            None => utils::get_config_dir(&dirname.to_string()),
        };
        let package_name = match yaml["packageName"].as_str() {
            Some(package_name) => Some(package_name.to_string()),
            None => None,
        };
        let dependencies = match yaml["dependencies"].as_sequence() {
            Some(dependencies) => dependencies
                .into_iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect::<Vec<String>>(),
            None => Vec::new(),
        };
        let files = match yaml["files"].as_sequence() {
            Some(files) => files
                .into_iter()
                .map(|x| File::new(&x, &source_dir, &target_dir))
                .collect::<Vec<File>>(),
            None => vec![File::new(
                &Value::String("".to_string()),
                &source_dir,
                &target_dir,
            )],
        };
        let custom_steps = match yaml["customSteps"].as_sequence() {
            Some(custom_steps) => custom_steps
                .into_iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect(),
            None => Vec::new(),
        };
        return Module {
            name: name.to_string(),
            source_dir: source_dir.to_string(),
            enabled: enabled,
            target_dir: target_dir,
            package_name: package_name,
            dependencies: dependencies,
            files: files,
            custom_steps: custom_steps,
        };
    }

    pub fn install(&self, install_command: &String) -> Result<(), String> {
        match &self.package_name {
            Some(package_name) => {
                info!("Installing {}...", package_name);
                let res = utils::cmd(&format!("{} {}", &install_command, package_name));
                match res {
                    Ok(_) => (),
                    Err(e) => warn!("{} failed to install: {}", package_name, e),
                }
            }
            None => (),
        };
        for dep in self.dependencies.iter() {
            info!("Installing {}...", dep);
            let res = utils::cmd(&format!("{} {}", &install_command, dep));
            match res {
                Ok(_) => (),
                Err(e) => warn!("{} failed to install: {}", dep, e),
            }
        }
        Ok(())
    }

    pub fn run_custom_steps(&self) -> Result<(), String> {
        for step in self.custom_steps.iter() {
            info!("Running custom step {}...", step);
            let res = utils::cmd(&step);
            match res {
                Ok(_) => (),
                Err(e) => error!("{} failed to run: {}", step, e),
            };
        }
        Ok(())
    }

    pub fn link_files(&self, variables: Vec<Variable>) -> Result<(), String> {
        for file in self.files.iter() {
            info!("Copying {}...", file.source);
            let res = file.link(variables.clone());
            match res {
                Ok(_) => (),
                Err(e) => error!("{} failed to copy: {}", file.source, e),
            };
        }
        Ok(())
    }
}
