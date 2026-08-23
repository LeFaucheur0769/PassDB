pub struct AppContext {
    pub db_location: String,
    pub import_dir: String,
    pub export_dir: String,
    // Add other shared state as needed
}

impl AppContext {
    pub fn new(import_dir: String, db_location: String,  export_dir: String) -> Self {
        Self {
            import_dir,
            db_location,
            export_dir,
        }
    }
}