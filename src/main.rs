// Import the mods

mod search;
mod sorter;
mod tui;

// Import clap to use arguments with PassDB

use std::{
    fs::{self},
    path::Path,
    vec,
};

use clap::{Arg, Parser, error::Result};

// Imports

use md5::digest::consts::True;
use yaml_rust2::{self};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of the config file
    #[arg(short, long, default_value = "passdb.yml")]
    config: String,

    /// Launch PassDB in intercative mod
    #[arg(short, long)]
    interactive: bool,

    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Disable the output
    #[arg(short, long)]
    quiet: bool,

    /// Specify an email address to verify
    #[arg(short, long)]
    email: String,

    /// Specify an import directory
    #[arg(long, default_value = "import/")]
    import: String,

    /// Specify an output file
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Debug)]
struct PassDB {
    debug: bool,
    db_location: String,
    create_db_in_toolFolder: bool,
    export_results_location: String,
    import_location: String,
    print_result_export_file: bool,
    check_if_valid_combolist: bool,
    nbr_of_check_per_file: i64,
    file_to_sort_location: String,
    fiel_to_sort_not_txt_files: String,
    file_urlloginpass_dir: String,
    addAddFile: bool,
    logs: Vec<String>,
}

impl PassDB {
    fn new() -> Self {
        let config_location = Args::parse().config;
        let read_config =
            fs::read_to_string(&config_location).expect("Failed to read the config location");

        let config = yaml_rust2::YamlLoader::load_from_str(&read_config).expect("Invalid YAML");

        PassDB {
            debug: config[0]["debug"].as_bool().unwrap(),
            db_location: config[0]["db_location"].as_str().unwrap().to_string(),
            create_db_in_toolFolder: config[0]["create_db_in_toolFolder"]
                .as_bool()
                .unwrap_or(true),
            export_results_location: config[0]["export_results_location"]
                .as_str()
                .unwrap_or("export/")
                .to_string(),
            import_location: config[0]["import_location"]
                .as_str()
                .unwrap_or("import/")
                .to_string(),
            print_result_export_file: config[0]["print_result_export_file"]
                .as_bool()
                .unwrap_or(false),
            check_if_valid_combolist: config[0]["check_if_valid_combolist"]
                .as_bool()
                .unwrap_or(true),
            nbr_of_check_per_file: config[0]["nbr_of_check_per_file"].as_i64().unwrap_or(10),
            file_to_sort_location: config[0]["file_to_sort_location"]
                .as_str()
                .unwrap_or("to_sort/")
                .to_string(),
            fiel_to_sort_not_txt_files: config[0]["fiel_to_sort_not_txt_files"]
                .as_str()
                .unwrap_or("to_sort/not_txt/")
                .to_string(),
            file_urlloginpass_dir: config[0]["file_urlloginpass_dir"]
                .as_str()
                .unwrap_or("to_sort/url_login_pass/")
                .to_string(),
            addAddFile: config[0]["addAddFile"].as_bool().unwrap_or(false),
            logs: vec![],
        }
    }

    fn test(&mut self) {
        println!("{:#?}", self);
        println!("{:#?}", self.debug);
    }

    fn run_menu(&mut self) {
        let _menu = tui::tui(
            self.import_location.clone(),
            self.db_location.clone(),
            self.file_to_sort_location.clone(),
            self.export_results_location.clone(),
        );
    }

    fn run_no_menu(&mut self, email: String) -> Result<()> {
        let mut search = search::search::Searcher::new(
            self.db_location.clone(),
            self.export_results_location.clone(),
            email,
        );

        search.search()?;
        Ok(())
    }

    fn ensure_dir(&mut self, path: &str) -> Result<bool, String> {
        if Path::new(path).is_dir() {
            self.logs.push(format!("Directory {} exists", path));
            Ok(true)
        } else {
            self.logs
                .push(format!("Directory {} missing, creating...", path));
            std::fs::create_dir_all(path)
                .map_err(|e| format!("Failed to create {}: {}", path, e))?;
            Ok(true)
        }
    }

    fn check_if_valid_or_create_dir(&mut self) -> Result<bool, String> {
        let paths = [
            self.db_location.clone(),
            self.export_results_location.clone(),
            self.import_location.clone(),
            self.file_to_sort_location.clone(),
        ];

        for path in paths.iter() {
            self.ensure_dir(path)?;
        }

        Ok(true)
    }

    fn run(&mut self) -> Result<(), String> {
        //self.check_if_valid_or_create_dir()?; // propagates Err

        if Args::parse().interactive {
            self.run_menu();
        } else {
            let email = Args::parse().email;
            self.run_no_menu(email);
        }
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut passdb = PassDB::new();
    passdb.run()?;
    Ok(())
}
