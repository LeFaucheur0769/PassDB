use md5::{self, Digest};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::result::Result::Ok;
use crate::log;
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
        let output_folder = db_location.to_string() + "/output/";

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
        // clone the hasher to manipulate it later without borrowing issues

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
            .open(&self.hashdb)?; // open the file using openoptions to open it in append mode or create the file if it doesn't exist

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
    //! This function is used to sort the file
    //! It accepts as input the path of the file to sort as &Path and db_dir as &str
    pub fn new(path: &Path, db_dir: &str) -> color_eyre::Result<Self> {
        //log!("Creating Sort {}", path.to_string_lossy());
        let file = File::open(path)?; // open the file to sort
        let total_size = std::fs::metadata(path)?.len(); // get the size of the file
        // Get just the file name as a String
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid file name"))?
            .to_string(); // get the file_name or return an error is the name if invalid
        let output_dir = format!("{}/sorted/", db_dir.trim_end_matches('/'));
        //log!("{}", output_dir);
        Ok(Sort {
            file,
            total_size,
            output_dir,
            file_name,
        })
    }
    /// Sort the file in an optimized and safe manner
    /// This function will sort the file in two passes:
    /// First pass: collect all unique groups
    /// Second pass: write each group to its file (one at a time)
    pub fn sort_optimised_safe(&mut self) -> color_eyre::Result<String> {
        //log!("Sort optimised safe");
        fs::create_dir_all(&self.output_dir)?;

        // First pass: collect all unique groups
        let mut groups: HashMap<String, Vec<String>> = HashMap::new(); // create a hashmap and vec<string group> to process the file
        let reader = BufReader::new(&self.file);
        // First pass: collect all unique groups
        //log!("First pass: collecting groups");

        for line_result in reader.split(b'\n') {
            let bytes = match line_result {
                Ok(b) => b,
                Err(e) => { log!("Read error: {}", e); continue; }
            };
            // Lossily convert - replaces invalid UTF-8 chars with ?
            let line = String::from_utf8_lossy(&bytes).into_owned();


            // Run the line through the checking process and return an empty line if invalid
            let cleaned_line = self.sorting_funct(line.as_str());
            // log!("Cleaned line : {}", cleaned_line);
            let trimmed = cleaned_line.trim(); // trime the line to remove blank lines and whitespaces

            // log!("trimmed : {}", trimmed);

            if trimmed.is_empty() {
                // log!("After trim, the line was empty");
                continue; // Next line if line is empty
            }

            // If the line was not empty
            let first_3 = trimmed.chars().take(3).collect::<String>();
            let sanitized = sanitize_filename(&first_3);
            // log!("Sanitized file name with 3 chars {}", sanitized);
            if sanitized.is_empty() {
                //log!("After sanitize, the file was empty");
                continue; // If all the characters pass the line
            }

            // Add the line to the corresponding group

            groups
                .entry(sanitized)
                .or_insert_with(Vec::new)
                .push(trimmed.to_string());
        }

        // Second pass: write each group to its file (one at a time)
        log!("Second pass: sorting");

        let mut file_count = 0;
        let total_groups = groups.len();

        log!("Wrote {} groups", total_groups);


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

    pub fn check_if_contains_url(&self, login: &str) -> bool {
        let url_parts_to_check = ["https://", "http://", "www."];
        for part_to_check in url_parts_to_check {
            if login.contains(part_to_check) {
                return true;
            }
        }
        false
    }
    pub fn sorting_funct(&self, login: &str) -> String {
        let mut cleaned;
        let separators = [" ", ":", ",", ";"];

        // Check if contains url parts and if yes clean them
        if self.check_if_contains_url(login) {
            cleaned = self.clean_url_in_login(login);
        } else {
            cleaned = login.to_string()
        }
        
        self.separator_function_tmp_name(cleaned.as_str())

    }

    pub fn clean_url_in_login(&self, login: &str) -> String {
        let mut cleaned: String;

        // removes the https:// to prevent issues with the :
        cleaned = login.replace("https://", "");

        //  removes the http:// to prevent issues with the :
        cleaned = cleaned.replace("http://", "");

        // If the login does not contain any strange formating, just add it to the db
        cleaned = cleaned.trim().to_string();

        cleaned
    }

    pub fn separator_function_tmp_name(&self, login: &str) -> String {
        // The available separators
        let list_separators = [':', ';', ',', ' '];
        let mut valid_separators: Vec<(char, u8)> = vec![];
        let mut output: String = "".to_string();

        // Check for separators
        for separator in list_separators {
            let separator_appearance = login.chars().filter(|x| *x == separator).count();
            // Return invalid format if no separators
            if separator_appearance == 0 {
                // println!("{} invalid format for '{}'", login, separator)
            }
            // Todo if the format is a valid lp
            if separator_appearance == 1 {
                // Add the separator and the number of appearance
                valid_separators.push((separator, separator_appearance as u8));
            }
            // Todo if the format is a valid ulp
            if separator_appearance == 2 {
                valid_separators.push((separator, separator_appearance as u8));
            }

            // Return invalid format if more than 2 separators
            // Might have to do something for the stealer logs
            if separator_appearance > 2 {
                // println!("{} invalid format for '{}'", login, separator)
            }
        }

        // Check if multiple separators and add the number to a vec of u8
        let appears_once_or_twice: Vec<u8> = valid_separators
            .iter()
            .filter(|(_, value)| *value == 1 || *value == 2)
            .map(|(_, value)| *value)
            .collect();

        // Check if only one separator appears once or twice
        if appears_once_or_twice.iter().count() != 1 {
            if valid_separators
                .iter()
                .filter(|(_, value)| *value == 1 || *value == 2)
                .count()
                == 1
                && valid_separators
                .iter()
                .filter(|(_, value)| *value > 2)
                .count()
                == 1
            {
                /*
                Todo
                    Considers that there is a strange thing but separators are still found so the line is good to import
                    return login
                */
                output = login.to_string();


            }
        } else {
            // There is only one separator that appears once or twice
            if appears_once_or_twice.iter().any(|&x| x == 2) {
                /*
                Todo
                    Considered as ulp
                */
                for (sep, c) in valid_separators {
                    if c == 2 {
                        let split = login.split(sep).collect::<Vec<&str>>();
                        if split.len() == 3 {
                            /*
                          Considers that the format of the import is ulp and so moving it to lpu
                          */
                            return(format!("{}:{}:{}", split[1], split[2], split[0]))

                        } else {

                            // println!("{} invalid format for '{}'", login, sep);
                        }

                    }
                }
            } else {
                /*
                Todo
                    considered as valid combo, good to import
                */
                return login.to_string()
            }
        }
        output
        // split at the separator if doable
    }

}
