use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek},
    path::PathBuf,
};

pub fn search() {
    println!("None");
}

pub struct Searcher {
    db_dir: String,
    export_dir: String,
    files: Vec<PathBuf>, // <- change here
    output: Vec<String>,
    email_to_search: String,
    total_bytes: u64,
    nbr_line: u64,
    current_pos: f64,
}

impl Searcher {
    pub fn new(
        db_dir: String,
        export_dir: String,
        email_to_search: String,
    ) -> Result<Self, color_eyre::Report> {
        let mut prefix = email_to_search.clone();
        if prefix.len() > 3 {
            prefix.truncate(3); // optional, max 3 chars
        }

        let export_path = PathBuf::from(std::env::current_dir().unwrap())
            .join(db_dir.clone())
            .join("sorted");

        // Match all files that start with the prefix
        let mut files = Vec::new();
        for entry in std::fs::read_dir(&export_path)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if file_name.starts_with(&prefix) && file_name.ends_with(".txt") {
                files.push(entry.path());
            }
        }

        if files.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "No files found with prefix '{}'",
                prefix
            ));
        }

        Ok(Searcher {
            db_dir: export_path.to_string_lossy().to_string(),
            export_dir,
            files,
            output: vec![],
            email_to_search,
            total_bytes: 0,
            nbr_line: 0,
            current_pos: 0.0,
        })
    }

    pub fn search(&mut self) -> std::io::Result<bool> {
        self.output.clear();
        self.nbr_line = 0;
        self.total_bytes = 0;
        self.current_pos = 0.0;

        for file_path in &self.files {
            let file = File::open(file_path)?;
            let mut reader = BufReader::new(file);
            loop {
                let mut buf = Vec::new();
                let bytes_read = reader.read_until(b'\n', &mut buf)?;

                if bytes_read == 0 {
                    break;
                }

                self.nbr_line += 1;

                let line = String::from_utf8_lossy(&buf);

                if line.contains(&self.email_to_search) {
                    self.output.push(line.trim_end().to_string());
                }

                self.current_pos += bytes_read as f64; // cumulative progress
            }
        }

        Ok(true)
    }

    pub fn update(&mut self) {}

    pub fn progress(&self) -> f64 {
        self.current_pos / self.total_bytes as f64
    }

    pub fn get_results(&self) -> &[String] {
        &self.output
    }
}
