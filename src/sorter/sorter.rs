use crate::sorter::parquet_sorter::Contact;
use crate::sorter::{parquet_sorter};
use color_eyre::eyre::eyre;
use color_eyre::{Result, eyre};
use md5::{self, Digest};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
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
    archive_name: String,
    archive_namedb: String,
}

impl HashFile {
    // Initialization of the variables used by the hasher
    pub fn new(path: &Path, db_location: &str) -> color_eyre::Result<Self> {
        let file = File::open(path)?; // Open the file specified in path
        let total_size = std::fs::metadata(path)?.len(); // Get the total size of the file to visualize the progress
        let hashdb = db_location.to_string() + "/hashdb"; // get the locatation of the hashdb file to then save the calculated hash of the file
        let _output_folder = db_location.to_string() + "/output/";
        let archive_name = sanitize_filename(path.file_name().unwrap().to_str().unwrap());
        let archive_namedb = db_location.to_string() + "/archive_namedb";

        Ok(HashFile {
            file,
            hasher: md5::Md5::new(),
            total_size,
            bytes_read: 0,
            hashdb,
            archive_name,
            archive_namedb,
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

        let mut archive_namedb = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.archive_namedb)?;
        archive_namedb.write_all(format!("{}\n", &self.archive_name).as_bytes())?;

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
        fs::create_dir_all(&self.output_dir)?;
        let reader = BufReader::new(&self.file);
        let separator = ':';

        for line_result in reader.split(b'\n') {
            let bytes = line_result?;
            let line = String::from_utf8_lossy(&bytes).into_owned();

            let cleaned_line = self.sorting_funct(&line);
            let trimmed = cleaned_line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Extract email and the rest (url:password)
            let (email, rest) = match self.search_if_email(trimmed, separator) {
                Ok(tuple) => tuple,
                Err(_) => {
                    // No email found – skip this line
                    continue;
                }
            };

            // Extract URL from the rest and get remaining (password)
            let (url, password_str) = match self.search_if_url(&rest, separator) {
                Ok(tuple) => tuple,
                Err(_) => {
                    // No URL found – maybe the line is just email:password
                    // We can treat the whole 'rest' as the password
                    (String::new(), rest)
                }
            };

            // Now password_str should be the password (or empty)
            let password = if password_str.is_empty() {
                None
            } else {
                Some(password_str)
            };

            // If URL was not found, we set it to None (already empty string, but we want Option)
            let url_opt = if url.is_empty() { None } else { Some(url) };

            // All other fields
            let username: Option<String> = None;
            let name: Option<String> = None;
            let other: Option<String> = None;

            let contact = Contact {
                email: Some(email),   // assuming email is String
                username,
                password,
                url: url_opt,
                name,
                other,
            };

            parquet_sorter::add_combo(contact).map_err(|e| color_eyre::eyre::eyre!(e))?;
        }

        Ok(format!("Finished sorting {}", self.file_name))
    }

    pub fn search_if_email(
        &self,
        line: &str,
        separator: char,
    ) -> Result<(String, String), eyre::Error> {
        let parts: Vec<&str> = line.split(separator).collect();

        // Find the email (the only part containing '@')
        if let Some(email_part) = parts.iter().find(|&&p| p.contains('@')) {
            let email = email_part.trim().to_string();

            // Build the remaining line from parts that do NOT contain '@'
            let remaining_parts: Vec<&str> = parts
                .iter()
                .filter(|&&p| !p.contains('@'))
                .map(|s| s.trim())
                .collect();
            let remaining = remaining_parts.join(&separator.to_string());

            Ok((email, remaining))
        } else {
            Err(eyre!("No email found in line: {}", line))
        }
    }

    pub fn search_if_url(
        &self,
        line: &str,
        separator: char,
    ) -> Result<(String, String), eyre::Error> {
        let parts: Vec<&str> = line.split(separator).collect();
        let url_parts_to_check = ["https://", "http://", "www."];

        for part in &url_parts_to_check {
            if let Some(url_part) = parts.iter().find(|p| p.contains(part)) {
                let url = url_part.trim().to_string();
                let remaining_parts: Vec<&str> = parts
                    .iter()
                    .filter(|p| !p.contains(part))
                    .map(|s| s.trim())
                    .collect();
                let remaining = remaining_parts.join(&separator.to_string());
                return Ok((url, remaining));
            }
        }

        Ok(("".to_string(), line.to_string()))
    }

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
        let cleaned;
        let _separators = [" ", ":", ",", ";"];

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
                            return format!("{}:{}:{}", split[1], split[2], split[0]) ;
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
                return login.to_string();
            }
        }
        output
        // split at the separator if doable
    }
}
