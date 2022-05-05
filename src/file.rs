use handlebars::Handlebars;
use serde_yaml::Value;
use std::fs;
use std::os::unix::fs::symlink;

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

    // Link or parse and copy file to the target path.
    pub fn copy(&self, variables: Value) -> Result<(), String> {
        match self.mkdir() {
            Ok(_) => {
                self.backup()?;
                self.rm()?;
                if self.to_parse {
                    match self.parse(variables) {
                        Ok(parsed) => self.write(&parsed),
                        Err(e) => Err(e),
                    }
                } else {
                    self.link()
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    // Backup file before overwriting it.
    // This will not backup soflinks to directories.
    fn backup(&self) -> Result<(), String> {
        let backup_path = self.target.clone() + ".bak";
        let _ = fs::copy(&self.target, &backup_path);
        Ok(())
    }
    fn parse(&self, variables: Value) -> Result<String, String> {
        let orig = fs::read_to_string(&self.source).expect("Failed to read file");
        let reg = Handlebars::new();
        return match reg.render_template(&orig, &variables) {
            Ok(parsed) => Ok(parsed),
            Err(e) => Err(e.to_string()),
        };
    }
    fn write(&self, content: &String) -> Result<(), String> {
        fs::write(&self.target, content).expect("Failed to write file");
        Ok(())
    }
    fn link(&self) -> Result<(), String> {
        symlink(&self.source, &self.target)
            .expect("Failed to link file");
        Ok(())
    }
    fn mkdir(&self) -> Result<(), std::io::Error> {
        let path = self.target.clone();
        let dir = if let Some((_, parts)) = path.split("/")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .split_last() {
            parts.join("/")
        } else {String::new()};
        fs::create_dir_all(dir)
    }
    fn rm(&self) -> Result<(), String> {
        let _ = fs::remove_file(&self.target);
        Ok(())
    }
}
