use clap::Parser;

mod config;
mod file;
mod module;
mod utils;

use config::Config;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Configuration file to use
    #[clap(short, long)]
    config: Option<String>,
    // Toggle installing packages
    #[clap(short, long)]
    install: bool,
    // Dry run (print config)
    #[clap(short, long)]
    dry_run: bool,
    #[clap(short, long)]
    pull: bool,
    // Modules to process, empty means all. Any module explicitely specified will be processed,
    // ignoring the value specified in the config file.
    modules: Vec<String>,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let file_path = match args.config {
        Some(path) => path,
        None => {
            let mut res = utils::get_config_dir(&"yrice".to_string());
            res.push_str("/config.yaml");
            res
        }
    };
    let config = Config::new(file_path.to_string(), args.modules);
    if args.dry_run {
        println!("{:#?}", config);
        Ok(())
    } else {
        config.run(args.install, args.pull)
    }
}
