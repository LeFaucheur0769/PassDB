use md5::{self, Digest};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::result::Result::Ok;

pub fn sanitize_filename(s: &str) -> String {
    //! Function used to sanitize_filename and prevent the creation of wrong files
    //!  Accept as input a str and return a sanitized string
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
    // Initialization of the variables used by the hasher
    pub fn new(path: &Path, db_location: &str) -> color_eyre::Result<Self> {
        let file = File::open(path)?; // Open the file specified in path
        let total_size = std::fs::metadata(path)?.len(); // Get the total size of the file to visualize the progress
        let hashdb = db_location.to_string() + "/hashdb"; // get the locatation of the hashdb file to then save the calculated hash of the file

        Ok(HashFile {
            file,
            hasher: md5::Md5::new(),
            total_size,
            bytes_read: 0,
            hashdb,
        })
    }
    /// Update the hash process of a file
    ///
    /// This function is used to update the hash process of a file by reading a chunk of it and updating the hash content.
    /// It returns a boolean indicating if the hash process finished or not.
    ///
    /// The function reads a chunk of the file (8MB maximum) to reduce the memory usage, then it updates the hash content using the bytes read.
    /// The total bytes read are stored to create a progress bar.
    pub fn update(&mut self) -> color_eyre::Result<bool> {
        // Create a buffer to store the bytes read from the file
        let mut buffer = vec![0u8; 8 * 1024 * 1024]; // 8 MB on the heap

        let bytes = self.file.read(&mut buffer)?; // Read a chunk of the file to reduce memory usage

        if bytes == 0 {
            // EOF
            return Ok(false);
        }

        // Update the hash content using the bytes read
        self.hasher.update(&buffer[..bytes]);

        // Add the bytes read to the total bytes read to allow the creation of a progress bar
        self.bytes_read += bytes as u64;
        Ok(true)
    }
    pub fn progress(&self) -> f64 {
        self.bytes_read as f64 / self.total_size as f64
        // Return the progress of the hash to use in a progress bar
    }

    /// Finalize the hash process and save the hash to the hashdb if not already present
    ///
    /// This function is used to finalize the hash process and save the hash to the hashdb file.
    /// It returns a tuple containing the hash calculated and a boolean indicating if the hash already existed in the hashdb.
    pub fn finalize(&self) -> color_eyre::Result<(String, bool)> {
        let cloned = self.hasher.clone();
        // clone the hasher to manipulate it later without borowing issues

        let result = cloned.finalize(); // get the result of
        // the hash to use it multiple times without rerunning the hash process
        let hash = format!("{:x}", result);

        if let Ok(file) = File::open(&self.hashdb) {
            // open the hashdb file
            let reader = BufReader::new(file); // create a bufreader to read the file
            for line in reader.lines() {
                // read each line of the file to check if it corresponds to the hash
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
            .open(&self.hashdb)?; // open the file using openoptions to open it in append mode or create the file if it doesn't exists

        hashdb.write_all(format!("{:x}\n", result).as_bytes())?; // append the hash and add a newline for the next hash

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
    //! Sort function
    //! This function is use to sort the file
    //! It accepts as input the path of the file to sort as &Path and db_dir as &str
    pub fn new(path: &Path, db_dir: &str) -> color_eyre::Result<Self> {
        let file = File::open(path)?; // open the file to sort
        let total_size = std::fs::metadata(path)?.len(); // get the size of the file
        // Get just the file name as a String
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid file name"))?
            .to_string(); // get the file_name or return an error is the name if invalid
        let output_dir =
            "/home/grimreaper/Desktop/DEV/Rust/project/PassDB/output/sorted/".to_string();
        // Hard-coded output dir needs to be changed later
        Ok(Sort {
            file,
            total_size,
            output_dir,
            file_name,
        })
    }
    /// Sort the file in an optimised and safe manner
    /// This function will sort the file in two passes:
    /// First pass: collect all unique groups
    /// Second pass: write each group to its file (one at a time)
    pub fn sort_optimised_safe(&mut self) -> color_eyre::Result<String> {
        fs::create_dir_all(&self.output_dir)?;

        // First pass: collect all unique groups
        let mut groups: HashMap<String, Vec<String>> = HashMap::new(); // create a hashmap and vec<string group> to process the file
        let reader = BufReader::new(&self.file);

        // First pass: collect all unique groups
        //println!("First pass: collecting groups...");
        for line_result in reader.lines() {
            let line = line_result?; // get the line and propagate the error if there is one
            let trimmed = line.trim(); // trime the line to remove blank lines and whitespaces
            if trimmed.is_empty() {
                continue; // Next line if line is empty
            }

            // If the line was not empty
            let first_3 = trimmed.chars().take(3).collect::<String>();
            let sanitized = sanitize_filename(&first_3);
            if sanitized.is_empty() {
                continue; // If all the characters pass the line
            }

            // Add the line to the corresponding group
            groups
                .entry(sanitized)
                .or_insert_with(Vec::new)
                .push(trimmed.to_string());
        }

        // Second pass: write each group to its file (one at a time)
        let mut file_count = 0;
        let total_groups = groups.len();

        // Second pass: write each group to its file (one at a time)
        for (i, (group_name, lines)) in groups.into_iter().enumerate() {
            if lines.is_empty() {
                continue;
            }

            let file_path = PathBuf::from(&self.output_dir).join(format!("{}.txt", group_name));
            let file = fs::OpenOptions::new()
                .append(true)
                .create(true) // Create if doesn't exist
                .open(file_path)?;
            let mut writer = BufWriter::new(file);

            // Write each line to the file
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
