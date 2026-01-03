use md5::{self, Digest};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::result::Result::Ok;

pub fn sanitize_filename(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

pub struct HashFile {
    file: File,
    hasher: md5::Md5,
    total_size: u64,
    bytes_read: u64,
    hashdb: String,
}

impl HashFile {
    pub fn new(path: &Path, db_location: &str) -> color_eyre::Result<Self> {
        let file = File::open(path)?;
        let total_size = std::fs::metadata(path)?.len();
        let hashdb = db_location.to_string() + "/hashdb";

        Ok(HashFile {
            file,
            hasher: md5::Md5::new(),
            total_size,
            bytes_read: 0,
            hashdb,
        })
    }
    pub fn update(&mut self) -> color_eyre::Result<bool> {
        let mut buffer = vec![0u8; 8 * 1024 * 1024]; // 8 MB on the heap
        let bytes = self.file.read(&mut buffer)?;
        if bytes == 0 {
            return Ok(false);
        }
        self.hasher.update(&buffer[..bytes]);
        self.bytes_read += bytes as u64;
        Ok(true)
    }

    pub fn progress(&self) -> f64 {
        self.bytes_read as f64 / self.total_size as f64
    }

    pub fn finalize(&self) -> color_eyre::Result<(String, bool)> {
        let cloned = self.hasher.clone(); // clone the hasher to manipulate
        // it later without borowing issues

        let result = cloned.finalize(); // get the result of
        // the hash to use it multiple times without rerunning the hash process
        let hash = format!("{:x}", result);

        if let Ok(file) = File::open(&self.hashdb) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line: String = line?;
                if line.trim() == hash {
                    return Ok((hash, true)); // return the hash to log it and true to specify
                    // that the hash already existed in the db
                }
            }
        }

        // the file was not found in the db so the db is opened in append mode and the hash it
        // formated then wrote to it
        let mut hashdb = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.hashdb)?;

        hashdb.write_all(format!("{:x}\n", result).as_bytes())?;

        Ok((hash, false)) // return the hash to log it and false to specify that the hash was not
        // present in the db and allow the sorting process to start
    }
}

pub struct Sort {
    file: File,
    total_size: u64,
    output_dir: String,
    file_name: String,
}

impl Sort {
    pub fn new(path: &Path, db_dir: &str) -> color_eyre::Result<Self> {
        let file = File::open(path)?;
        let total_size = std::fs::metadata(path)?.len(); // get the size of the file
        // Get just the file name as a String
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid file name"))?
            .to_string();
        let output_dir =
            "/home/grimreaper/Desktop/DEV/Rust/project/PassDB/output/sorted/".to_string();
        Ok(Sort {
            file,
            total_size,
            output_dir,
            file_name,
        })
    }
    pub fn sort_optimised_safe(&mut self) -> color_eyre::Result<String> {
        fs::create_dir_all(&self.output_dir)?;

        // First pass: collect all unique groups
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();
        let reader = BufReader::new(&self.file);

        //println!("First pass: collecting groups...");
        for line_result in reader.lines() {
            let line = line_result?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let first_3 = trimmed.chars().take(3).collect::<String>();
            let sanitized = sanitize_filename(&first_3);
            if sanitized.is_empty() {
                continue;
            }

            groups
                .entry(sanitized)
                .or_insert_with(Vec::new)
                .push(trimmed.to_string());
        }

        //println!("Found {} unique groups", groups.len());

        // Second pass: write each group to its file (one at a time)
        let mut file_count = 0;
        let total_groups = groups.len();

        for (i, (group_name, lines)) in groups.into_iter().enumerate() {
            if lines.is_empty() {
                continue;
            }

            // println!(
            //     "Writing group {}/{}: {} ({} lines)",
            //     i + 1,
            //     total_groups,
            //     group_name,
            //     lines.len()
            // );

            let file_path = PathBuf::from(&self.output_dir).join(format!("{}.txt", group_name));
            let file = fs::OpenOptions::new()
                .append(true)
                .create(true) // Create if doesn't exist
                .open(file_path)?;
            let mut writer = BufWriter::new(file);

            for line in lines {
                writeln!(writer, "{}", line)?;
            }

            writer.flush()?;
            file_count += 1;
        }

        Ok(format!(
            "Finished sorting {} (created {} sorted files)",
            self.file_name, file_count
        ))
    }

    // pub fn progress() -> f64 {
    // 0.0
    // }

    // pub fn finalize(&mut self) -> color_eyre::Result<String> {
    // Ok("Succesfull".to_string())
    // }
}
