extern crate serde_yaml;
extern crate resolve_path;
use serde_yaml::Value;
use resolve_path::PathResolveExt;
use std::path::Path;

use crate::file::File;
use crate::utils;
use crate::variable::Variable;

#[derive(Debug)]
pub struct Module {
    name: String,
    package_name: Option<String>,
    dependencies: Vec<String>,
    files: Vec<File>,
    pre_steps: Vec<String>,
    post_steps: Vec<String>,
    requires: Vec<String>
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
        let pre_steps = match yaml["preSteps"].as_sequence() {
            Some(pre_steps) => pre_steps
                .into_iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect(),
            None => Vec::new(),
        };
        let post_steps = match yaml["postSteps"].as_sequence() {
            Some(post_steps) => post_steps
                .into_iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect(),
            None => Vec::new(),
        };
        let requires = match yaml["requires"].as_sequence() {
            Some(requires) => requires
                .into_iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect(),
            None => Vec::new(),
        };
        return Module {
            name: name.to_string(),
            package_name: package_name,
            dependencies: dependencies,
            files: files,
            pre_steps: pre_steps,
            post_steps: post_steps,
            requires: requires
        };
    }

    pub fn get_name(&self) -> &String {
        return &self.name;
    }


    pub fn get_requires(&self) -> &Vec<String> {
        return &self.requires;
    }
    // Install the system package and dependencies
    pub fn install(&self, install_command: &String) -> Result<(), String> {
        match &self.package_name {
            Some(package_name) => {
                println!("\tInstalling {}...", package_name);
                let res = utils::cmd(&format!("{} {}", &install_command, package_name));
                let _ = match res {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("{} failed to install: {}", package_name, e)),
                };
            }
            None => (),
        };
        for dep in self.dependencies.iter() {
            println!("\tInstalling {}...", dep);
            let res = utils::cmd(&format!("{} {}", &install_command, dep));
            let _ = match res {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{} failed to install: {}", dep, e)),
            };
        }
        Ok(())
    }

    // Run the custom steps
    // These will be ran in the order they are defined
    pub fn run_custom_steps(&self, pre: bool) -> Result<(), String> {
        let steps = if pre {
            &self.pre_steps
        } else {
            &self.post_steps
        };
        for step in steps.iter() {
            println!("\tRunning custom step {}...", step);
            let res = utils::cmd(&step);
            let _ = match res {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{} failed to run: {}", step, e)),
            };
        }
        Ok(())
    }

    // Copy the files to the target directory
    // If the file needs to be parsed, it will be parsed and written 
    // If the file does not need to be parsed, it will be softlinked
    pub fn link_files(&self, variables: Vec<Variable>) -> Result<(), String> {
        for file in self.files.iter() {
            println!("\tCopying {}...", file.get_source());
            let _ = match file.copy(variables.clone()) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{} failed to copy: {}", file.get_source(), e)),
            };
        }
        Ok(())
    }
}
