use crate::module::Module;
use serde_yaml::Value;

#[derive(Debug)]
pub struct Globals {
    pub install_command: String,
    pub dotfiles: String,
}

#[derive(Debug)]
pub struct Config {
    pub filename: String,
    pub globals: Globals,
    pub variables: Value,
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
            variables: Value::Null,
            modules: Vec::new(),
        };

        config
            .load(&enabled_modules)
            .expect("Failed to load config");

        return config;
    }

    // Load globals from "config" key in yaml
    fn load_globals(&mut self, globals: Value) -> Result<(), String> {
        for (key, value) in globals.as_mapping().unwrap().iter() {
            match key.as_str() {
                Some("installCommand") => {
                    self.globals.install_command = value.as_str().unwrap().to_string();
                }
                Some("dotfiles") => {
                    self.globals.dotfiles = value.as_str().unwrap().to_string();
                }
                Some(k) => {
                    println!("[Warn] Unknown global key: {}", k);
                }
                _ => {
                    panic!("Unknown error parsing globals")
                }
            }
        }
        Ok(())
    }

    // Load modules from "modules" key in yaml
    fn load_modules(&mut self, modules: Value, enabled_modules: &Vec<String>) {
        let mut handled = Vec::new();
        let all = enabled_modules.len() == 0;
        for (raw_key, value) in modules.as_mapping().unwrap().iter() {
            let key = &raw_key.as_str().unwrap().to_string();
            let enabled = match value["enabled"].as_bool() {
                Some(b) => b,
                None => true,
            };
            if (all && enabled) || enabled_modules.contains(key) {
                let module = Module::new(key, value, &self.globals.dotfiles);
                for req in module.get_requires().iter() {
                    if !handled.contains(req) {
                        match modules.get(req) {
                            Some(m) => {
                                self.modules.push(Module::new(
                                    req,
                                    &m.clone(),
                                    &self.globals.dotfiles,
                                ));
                                handled.push(req.to_string());
                            }
                            None => {
                                println!(
                                    "\t[Warn] Module {} requires {} but it is was not found",
                                    key, req
                                );
                            }
                        }
                    }
                }
                self.modules
                    .push(Module::new(key, value, &self.globals.dotfiles));
                handled.push(key.to_string());
            }
        }
    }

    // Load config from yaml file
    pub fn load(&mut self, enabled_modules: &Vec<String>) -> Result<(), String> {
        let file = std::fs::File::open(self.filename.clone());
        if file.is_err() {
            return Err(format!("Could not open config file: {}", self.filename));
        }
        let yaml: Value = serde_yaml::from_reader(file.unwrap()).unwrap();

        for (key, value) in yaml.as_mapping().unwrap().iter() {
            match key.as_str().unwrap() {
                "global" => {
                    self.load_globals(value.clone())?;
                }
                "variables" => {
                    self.variables = value.clone();
                }
                "modules" => {
                    self.load_modules(value.clone(), enabled_modules);
                }
                k => {
                    println!("[Warn] Unknown toplevel key: {}", k);
                }
            }
        }
        println!("[Info] Loaded config: {}", self.filename);
        return Ok(());
    }

    // Run YDots
    pub fn run(&self, install: bool) -> Result<(), String> {
        let n = self.modules.len();
        let mut i = 1;
        for m in self.modules.iter() {
            println!("[{}/{}] {}...", i, n, m.get_name());
            i += 1;
            if install {
                m.install(&self.globals.install_command)?;
            }
            m.run_custom_steps(true)?;
            m.link_files(self.variables.clone())?;
            m.run_custom_steps(false)?;
            println!("[OK]");
        }
        Ok(())
    }
}
