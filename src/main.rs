// Allowed issues
// #[allow(unused_variables)]
// #[allow(noop_method_call)]

// Import the mods

mod logging;
mod search;
mod sorter;
mod test_utils;
mod tools;
mod tui;

// Import clap to use arguments with PassDB

use std::{
    env,
    fs::{self},
    path::Path,
    path::PathBuf,
    vec,
};

use clap::{CommandFactory, Parser, error::Result};

// Imports

use yaml_rust2::{self};

use crate::clean_files::CleanLine;
use crate::logging::test;
use crate::tools::clean_files;
use crate::tui::test_tui::test_fun;

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
    email: Option<String>,

    /// Specify an import directory
    #[arg(long, default_value = "import/")]
    import: String,

    /// Specify an output file
    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    test: bool,
}

fn get_default_import_dir() -> PathBuf {
    let exe_dir = env::current_exe()
        .expect("Failed to get exe path")
        .parent()
        .expect("Failed to get exe directory")
        .to_path_buf();
    exe_dir.join("import")
}

#[derive(Debug)]
struct PassDB {
    debug: bool,
    db_location: String,
    create_db_in_tool_folder: bool,
    export_results_location: String,
    import_location: String,
    print_result_export_file: bool,
    check_if_valid_combolist: bool,
    nbr_of_check_per_file: i64,
    file_to_sort_location: String,
    file_to_sort_not_txt_files: String,
    file_urlloginpass_dir: String,
    add_file: bool,
    logs: Vec<String>,
}

impl PassDB {
    fn new() -> Self {
        let config_location = Args::parse().config;
        let read_config =
            fs::read_to_string(&config_location).expect("Failed to read the config location");
        let config = yaml_rust2::YamlLoader::load_from_str(&read_config).expect("Invalid YAML");

        // Get either the config path or default, as PathBuf
        let import_path = config[0]["import_location"]
            .as_str()
            .map(PathBuf::from)
            .unwrap_or_else(get_default_import_dir);

        // Convert to absolute path
        let import_location = import_path.canonicalize().unwrap_or(import_path); // fallback if folder doesn't exist yet

        PassDB {
            debug: config[0]["debug"].as_bool().unwrap(),
            db_location: config[0]["db_location"].as_str().unwrap().to_string(),
            create_db_in_tool_folder: config[0]["create_db_in_toolFolder"]
                .as_bool()
                .unwrap_or(true),
            export_results_location: config[0]["export_results_location"]
                .as_str()
                .unwrap_or("export/")
                .to_string(),
            import_location: import_location.to_string_lossy().to_string(), // full path
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
            file_to_sort_not_txt_files: config[0]["fiel_to_sort_not_txt_files"]
                .as_str()
                .unwrap_or("to_sort/not_txt/")
                .to_string(),
            file_urlloginpass_dir: config[0]["file_urlloginpass_dir"]
                .as_str()
                .unwrap_or("to_sort/url_login_pass/")
                .to_string(),
            add_file: config[0]["add_file"].as_bool().unwrap_or(false),
            logs: vec![],
        }
    }
    fn test(
        &mut self,
        import_location: String,
        db_location: String,
        file_to_sort_location: String,
        export_results_location: String,
    ) -> color_eyre::Result<()> {
        color_eyre::install()?;
        println!("running the module test");
        test_fun()?;

        // let sorter_test = sorter::sorter::Sort::new(
        //     Path::new("/home/grimreaper/Desktop/DEV/Rust/project/PassDB/test.txt"),
        //     &self.db_location,
        // );
        // logging::test::new();
        // println!("{}", sorter_test.unwrap().sort_optimised_safe().unwrap());
        Ok(())
    }

    fn run_menu(&mut self) {
        let _menu = tui::tui(
            self.import_location.clone(),
            self.db_location.clone(),
            self.file_to_sort_location.clone(),
            self.export_results_location.clone(),
        );
        let _clean = CleanLine::new("ttt");
    }

    fn run_no_menu(&mut self, email: String) -> Result<()> {
        let mut search = match search::search::Searcher::new(
            self.db_location.clone(),
            self.export_results_location.clone(),
            email,
        ) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to initialize search: {e}");
                return Ok(()); // or return Err(clap::Error::raw(...)) if you want to propagate to clap
            }
        };

        search.search()?;

        Ok(())
    }

    fn run(&mut self) -> color_eyre::Result<()> {
        let args = Args::parse();

        if args.interactive {
            self.run_menu();
            Ok(())
        } else if args.test {
            self.test(
                self.import_location.clone(),
                self.db_location.clone(),
                self.file_to_sort_location.clone(),
                self.export_results_location.clone(),
            )?;
            Ok(())
        } else {
            let email = match args.email {
                Some(email) => email,
                None => {
                    Args::command().print_help().unwrap();
                    println!();
                    std::process::exit(1);
                }
            };

            self.run_no_menu(email)?;
            Ok(())
        }
    }
}

fn main() -> color_eyre::Result<()> {
    let mut passdb = PassDB::new();
    passdb.run()?;
    Ok(())
}
