// Import the mods

mod search;
mod sorter;
mod tui;

// Import clap to use arguments with PassDB

use std::{
    any::Any,
    fmt::format,
    fs::{self, OpenOptions},
    io::{self, read_to_string},
    path::Path,
    vec,
};

use clap::{Parser, builder::Str};

// Imports

use yaml_rust2::{self, YamlEmitter, emitter};

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

    fn run(&mut self) {
        let _menu = tui::tui();
    }

    fn check_if_valid_application_dir(&mut self) -> Result<bool, String> {
        let mut exists = true;

        if Path::new(&self.db_location).is_dir() {
            self.logs
                .push(format!("Directory {} exists", self.db_location));
        } else {
            self.logs
                .push(format!("Directory {} does not exist", self.db_location));
            exists = false;
        }

        if Path::new(&self.export_results_location).is_dir() {
            self.logs
                .push(format!("Directory {} exists", self.export_results_location));
        } else {
            self.logs.push(format!(
                "Directory {} does not exists",
                self.export_results_location
            ));
            exists = false;
        }

        if Path::new(&self.import_location).is_dir() {
            self.logs
                .push(format!("Directory {} exists", self.import_location));
        } else {
            self.logs.push(format!(
                "Directory {} does not exists",
                self.import_location
            ));
            exists = false;
        }

        if Path::new(&self.file_to_sort_location).is_dir() {
            self.logs
                .push(format!("Directory {} exists", self.file_to_sort_location));
        } else {
            self.logs.push(format!(
                "Directory {} does not exists",
                self.file_to_sort_location,
            ));
            exists = false;
        }

        if exists { Ok(true) } else { Ok(false) }
    }
}

fn main() {
    let mut passdb = PassDB::new();
    passdb.run();
}

fn main1() {
    let mut args = Args::parse();

    if args.output.is_none() {
        args.output = Some(format!("{}.txt", args.email));
    }
    let menu = tui::tui();

    println!("Email {}", args.email);
    if let Some(output) = &args.output {
        println!("Output file {}", output);
    } else {
        println!("No output file");
    }
}
