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
    file: File,
    output: Vec<String>,
    email_to_search: String,
    total_bytes: u64,
    nbr_line: u64,
    current_pos: f64,
}

impl Searcher {
    pub fn new(db_dir: String, export_dir: String, email_to_search: String) -> Self {
        let mut first_3_letters = email_to_search.clone();
        let export_path = PathBuf::from(std::env::current_dir().unwrap())
            .join(db_dir.clone())
            .join("sorted");
        first_3_letters.truncate(3);

        Searcher {
            db_dir: export_path.to_string_lossy().to_string(),
            export_dir,
            file: File::open(format!(
                "{}/{}.txt",
                export_path.to_string_lossy(),
                first_3_letters
            ))
            .unwrap(),
            output: vec![],
            email_to_search,
            total_bytes: 0,
            nbr_line: 0,
            current_pos: 0.0,
        }
    }

    pub fn search(&mut self) -> std::io::Result<bool> {
        self.total_bytes = self.file.metadata()?.len(); // total file size in bytes
        let mut reader = BufReader::new(&self.file);
        let mut nbr_line = 0u64;

        self.output.clear();

        loop {
            let mut buf = String::new();
            let bytes_read = reader.read_line(&mut buf)?;
            if bytes_read == 0 {
                break; // EOF
            }

            nbr_line += 1;

            if buf.contains(&self.email_to_search) {
                self.output.push(buf.trim_end().to_string());
            }

            // Here’s the ratio: bytes_read_so_far / total_bytes
            self.current_pos = reader.stream_position().unwrap() as f64; // how many bytes have been read

            // You can log or display this;

            // progress
            self.progress();
        }

        self.nbr_line = nbr_line;
        for i in self.output.clone() {
            println!("{}", i);
        }
        Ok(true)
    }

    pub fn update(&mut self) {}

    pub fn progress(&self) -> f64 {
        self.current_pos / self.total_bytes as f64
    }
}
