use std::env::current_dir;
use std::fs;
use std::fs::remove_dir;
use std::path::PathBuf;

pub fn check_for_config_folders() -> bool {
    use std::fs::create_dir_all;
    use std::path::Path;
    use yaml_rust2::{self};
    let config_file = "passdb.yml";
    let read_config =
        fs::read_to_string(&config_file).expect("Failed to read the config location");
    let config = yaml_rust2::YamlLoader::load_from_str(&read_config).expect("Invalid YAML");
    let current_dir = current_dir().unwrap().canonicalize().unwrap();

    // Check for db_location
    let db_location_path = current_dir.join(config[0]["db_location"].as_str().unwrap());
    if !Path::new(&db_location_path).is_dir() {
        // Creating the db folder
        println!("No db folder has been found, creating a new one");
        create_dir_all(&db_location_path).unwrap();
    }
    // Check for import_location folder
    let import_location_path = current_dir.join(config[0]["import_location"].as_str().unwrap());
    if !Path::new(&import_location_path).is_dir() {
        println!("No import location has been found, creating a new one");
        create_dir_all(&import_location_path).unwrap();
    }
    // Check for file_to_sort_location folder
    let file_to_sort_location_path =
        current_dir.join(config[0]["file_to_sort_location"].as_str().unwrap());
    if !Path::new(&file_to_sort_location_path).is_dir() {
        println!("No file_to_sort_location has been found, creating a new one");
        create_dir_all(&file_to_sort_location_path).unwrap();
    }
    crate::test_utils::test_write::cleanup_test_filters(db_location_path, import_location_path, file_to_sort_location_path, current_dir);
    true
}

/// Function used to clean after check_for_config_folders
pub fn cleanup_test_filters(
    db_location_path: PathBuf,
    import_location_path: PathBuf,
    file_to_sort_location_path: PathBuf,
    current_dir: PathBuf,
) {
    let folders_to_clean = [db_location_path, import_location_path, file_to_sort_location_path].to_vec();
    if current_dir.to_string_lossy().contains("RustRover") {
        for folder in folders_to_clean.iter() {
            remove_dir(folder).unwrap();
        }
    }
}