use std::path::Path;

pub fn check_config_file_exists(config_path: String) -> color_eyre::Result<bool> {
    if Path::new(config_path.as_str()).exists() {
        return Ok(true);
    }
    Ok(false)
}