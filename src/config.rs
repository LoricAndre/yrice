extern crate serde_yaml;
use log::{warn, error, info};
use serde_yaml::Value;

use crate::module::Module;
use crate::variable::Variable;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Globals {
    pub install_command: String,
    pub dotfiles: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Config {
    pub filename: String,
    pub globals: Globals,
    pub variables: Vec<Variable>,
    pub modules: Vec<Module>,
}

impl Config {
    pub fn new(filename: String, enabled_modules: Vec<String>) -> Config {
        let mut config = Config {
            filename,
            globals: Globals {
                install_command: String::from(""),
                dotfiles: String::from(""),
            },
            variables: Vec::new(),
            modules: Vec::new(),
        };

        config.load(&enabled_modules);

        return config;
    }

    fn load_globals(&mut self, globals: Value) {
        for (key, value) in globals.as_mapping().unwrap().iter() {
            match key.as_str() {
                Some("installCommand") => {
                    self.globals.install_command = value.as_str().unwrap().to_string();
                }
                Some("dotfiles") => {
                    self.globals.dotfiles = value.as_str().unwrap().to_string();
                }
                Some(k) => {
                    warn!("Unknown global key: {}", k);
                }
                _ => {
                    panic!("Unknown error parsing globals");
                }
            }
        }
    }

    fn load_variables(&mut self, variables: Value) {
        for (key, value) in variables.as_mapping().unwrap().iter() {
            let variable = Variable::new(
                key.as_str().unwrap().to_string(),
                value.as_str().unwrap().to_string(),
            );
            self.variables.push(variable);
        }
    }

    fn load_modules(&mut self, modules: Value, enabled_modules: &Vec<String>) {
        let all = enabled_modules.len() == 0;
        for (raw_key, value) in modules.as_mapping().unwrap().iter() {
            let key = &raw_key.as_str().unwrap().to_string();
            let enabled = match value["enabled"].as_bool() {
                Some(b) => b,
                None => true
            };
            if (all && enabled) || enabled_modules.contains(key) {
                self.modules
                    .push(Module::new(key, value, &self.globals.dotfiles));
            }
        }
    }

    pub fn load(&mut self, enabled_modules: &Vec<String>) {
        let file = std::fs::File::open(self.filename.clone());
        if file.is_err() {
            error!("Could not open config file: {}", self.filename);
            return ();
        }
        let yaml: Value = serde_yaml::from_reader(file.unwrap()).unwrap();

        for (key, value) in yaml.as_mapping().unwrap().iter() {
            match key.as_str().unwrap() {
                "global" => {
                    self.load_globals(value.clone());
                }
                "variables" => {
                    self.load_variables(value.clone());
                }
                "modules" => {
                    self.load_modules(value.clone(), enabled_modules);
                }
                k => {
                    warn!("Unknown key: {}", k);
                }
            }
        }
        info!("Loaded config: {}", self.filename);
    }

    pub fn run(&self, install: bool) -> Result<(), String> {
        let n = self.modules.len();
        let mut i = 1;
        for m in self.modules.iter() {
            print!("[{}/{}] {}...", i, n, m.name);
            i += 1;
            if install {
                m.install(&self.globals.install_command)?;
            }
            m.run_custom_steps()?;
            m.link_files(self.variables.clone())?;
            println!(" [OK]");
        }
        Ok(())
    }
}
