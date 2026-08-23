use crate::logging::LogEntry;
use crate::sorter::parquet_sorter;
use color_eyre::eyre::eyre;
use color_eyre::{Result, eyre};
use duckdb::Connection;
use md5::{self, Digest};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::result::Result::Ok;
use std::sync::mpsc::Sender;
#[derive(Debug, Clone)]
pub struct Contact {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub name: Option<String>,
    pub other: Option<String>,
    pub origin: Option<String>,
}

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

    pub fn sort_into_db(
        &mut self,
        conn: &Connection,
        log_tx: &Sender<LogEntry>,
    ) -> color_eyre::Result<String> {
        use duckdb::{Appender, ToSql};
        use std::io::BufRead;
        use std::time::Instant;

        fs::create_dir_all(&self.output_dir)?;
        let reader = BufReader::new(&self.file);
        let separator = ':';

        conn.execute("PRAGMA memory_limit='4GB';", [])?;
        conn.execute("PRAGMA threads=4;", [])?;
        conn.execute("PRAGMA temp_directory='/tmp/duckdb_temp';", [])?;

        let start_time = Instant::now();
        let mut total_rows = 0;
        let mut skipped_rows = 0;

        fn clean_phone(s: &str) -> String {
            s.chars().filter(|c| c.is_ascii_digit()).collect()
        }

        fn is_french_style(line: &str) -> bool {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            parts.len() >= 4 && (parts[0].contains("M.") || parts[0].contains("Mme."))
        }

        conn.execute("BEGIN TRANSACTION", [])?;
        let mut appender = conn.appender("contacts")?;

        const FLUSH_INTERVAL: usize = 100_000;
        const COMMIT_INTERVAL: usize = 5_000_000;

        let mut rows_since_flush = 0;
        let mut rows_since_commit = 0;

        // --- CSV header detection state ---
        let mut header_detected = false;
        let mut email_idx = usize::MAX;
        let mut phone_idx = usize::MAX;
        let mut username_idx = usize::MAX;
        let mut name_idx = usize::MAX;
        let mut password_idx = usize::MAX;

        // --- CSV headers
        let mut separator: char = ':';
        let mut headers: Vec<String> = Vec::new();

        for line_result in reader.split(b'\n') {
            let bytes = line_result?;
            let line = String::from_utf8_lossy(&bytes);
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // -------- Helper to append a single row (avoids code duplication) --------
            fn append_row(
                appender: &mut Appender,
                email: &str,
                username: &str,
                password: &str,
                url: &str,
                name: &str,
                other: &str,
                origin: &str,
            ) -> Result<(), duckdb::Error> {
                appender.append_row(&[
                    &email as &dyn ToSql,
                    &username as &dyn ToSql,
                    &password as &dyn ToSql,
                    &url as &dyn ToSql,
                    &name as &dyn ToSql,
                    &other as &dyn ToSql,
                    &origin as &dyn ToSql,
                ])
            }

            let mut parsed = false;

            // -------- 0. JSON detection (object or array) --------
            let is_json = trimmed.starts_with('{') || trimmed.starts_with('[');
            if is_json && !parsed {
                // Clean BOM if any
                let clean = trimmed.trim_start_matches('\u{FEFF}').trim();
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(clean) {
                    // Helper to extract fields from a JSON object
                    let extract_fields = |obj: &serde_json::Value| -> (String, String, String, String, String, String) {
                        let extract = |keys: &[&str]| -> String {
                            for key in keys {
                                if let Some(val) = obj.get(key).and_then(|v| v.as_str()) {
                                    if !val.is_empty() {
                                        return val.to_string();
                                    }
                                }
                            }
                            String::new()
                        };
                        let email_val = extract(&["email", "Email", "mail", "emailAddress"]);
                        let phone_raw = extract(&["mobilePhone", "phone", "telephone", "PhoneNumber", "cell"]);
                        let full_name = extract(&["fullName", "FullName", "name", "Name"]);
                        let surname = extract(&["surname", "lastName", "LastName"]);
                        let first_name = extract(&["firstName", "FirstName"]);
                        let address = extract(&["address", "Address", "street"]);
                        let zip = extract(&["zipCode", "postalCode"]);
                        let city = extract(&["city", "City", "town"]);

                        let phone_clean = clean_phone(&phone_raw);
                        let email = if !email_val.is_empty() {
                            email_val
                        } else if !phone_clean.is_empty() {
                            phone_clean.clone()
                        } else {
                            String::new()
                        };
                        let name = if !full_name.is_empty() {
                            full_name
                        } else if !surname.is_empty() && !first_name.is_empty() {
                            format!("{} {}", surname, first_name)
                        } else if !surname.is_empty() {
                            surname.clone()
                        } else {
                            String::new()
                        };
                        let other = format!(
                            "Surname: {}, FirstName: {}, Address: {} {} {}, Phone: {}",
                            surname, first_name, address, zip, city, phone_clean
                        );
                        (email, String::new(), String::new(), String::new(), name, other)
                    };

                    match value {
                        serde_json::Value::Array(arr) => {
                            // Process each object in the array
                            for obj in arr {
                                if let serde_json::Value::Object(_) = obj {
                                    let (email, username, password, url, name, other) =
                                        extract_fields(&obj);
                                    if !email.is_empty() || !username.is_empty() || !name.is_empty()
                                    {
                                        append_row(
                                            &mut appender,
                                            &email,
                                            &username,
                                            &password,
                                            &url,
                                            &name,
                                            &other,
                                            &self.file_name,
                                        )?;
                                        total_rows += 1;
                                        rows_since_flush += 1;
                                        rows_since_commit += 1;

                                        // Flush and commit logic (same as below)
                                        if rows_since_flush >= FLUSH_INTERVAL {
                                            appender.flush()?;
                                            rows_since_flush = 0;
                                        }
                                        if rows_since_commit >= COMMIT_INTERVAL {
                                            appender.flush()?;
                                            conn.execute("COMMIT", [])?;
                                            conn.execute("BEGIN TRANSACTION", [])?;
                                            rows_since_commit = 0;
                                            let elapsed = start_time.elapsed().as_secs_f64();
                                            let rate = total_rows as f64 / elapsed;
                                            let _ = log_tx.send(LogEntry::info(format!(
                                                "Committed {} rows, {:.0} rows/sec",
                                                total_rows, rate
                                            )));
                                        }
                                        parsed = true;
                                    }
                                }
                            }
                        }
                        serde_json::Value::Object(_) => {
                            // Single object
                            let (email, username, password, url, name, other) =
                                extract_fields(&value);
                            if !email.is_empty() || !username.is_empty() || !name.is_empty() {
                                append_row(
                                    &mut appender,
                                    &email,
                                    &username,
                                    &password,
                                    &url,
                                    &name,
                                    &other,
                                    &self.file_name,
                                )?;
                                total_rows += 1;
                                rows_since_flush += 1;
                                rows_since_commit += 1;
                                parsed = true;
                            }
                        }
                        _ => {} // ignore other types
                    }
                }
                // If we parsed any JSON, skip other parsers for this line
                if parsed {
                    // Log progress periodically (already done after all rows)
                    // We'll handle logging later to avoid duplication.
                    // But we need to flush/commit checks already inside.
                    // We'll skip the rest of this line and continue.
                    if total_rows % 1_000_000 == 0 && total_rows > 0 {
                        appender.flush()?;
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let rate = total_rows as f64 / elapsed;
                        let _ = log_tx.send(LogEntry::info(format!(
                            "Parsed {} rows (skipped: {}), {:.0} rows/sec",
                            total_rows, skipped_rows, rate
                        )));
                    }
                    continue;
                }
            }

            // -------- 1. CSV header detection (only if not JSON) --------

            if trimmed.contains(',') && !trimmed.contains(';') && !trimmed.contains('\t') {
                separator = ',';
            } else if trimmed.contains(';') && !trimmed.contains(',') && !trimmed.contains('\t') {
                separator = ';';
            } else if trimmed.contains('\t') && !trimmed.contains(',') && !trimmed.contains(';') {
                separator = '\t';
            } else {
                // Mixed delimiters: count which appears more often
                let comma_count = trimmed.chars().filter(|&c| c == ',').count();
                let semi_count = trimmed.chars().filter(|&c| c == ';').count();
                let tab_count = trimmed.chars().filter(|&c| c == '\t').count();
                if semi_count >= comma_count && semi_count >= tab_count {
                    separator = ';';
                } else if comma_count >= semi_count && comma_count >= tab_count {
                    separator = ',';
                } else {
                    separator = '\t';
                }
            }

            if !header_detected && !parsed && trimmed.contains(separator) {
                let raw_headers: Vec<&str> = trimmed.split(separator).map(|s| s.trim()).collect();
                // Check if this looks like a header
                let has_user_id = raw_headers.iter().any(|&h| h == "user_id");
                let has_email = raw_headers
                    .iter()
                    .any(|&h| h == "email" || h == "mail" || h == "ODQP_LB_MAIL_CONTACT");
                let has_username = raw_headers.iter().any(|&h| h == "username");
                let has_nickname = raw_headers.iter().any(|&h| h == "nickname" || h == "name");
                let has_mobile = raw_headers
                    .iter()
                    .any(|&h| h == "mobile_phone" || h == "phone" || h == "ODQP_LB_TEL_CONTACT");
                let has_password = raw_headers.iter().any(|&h| h == "password" || h == "hash");

                // Include password detection to be safe
                if has_user_id
                    || has_email
                    || has_username
                    || has_nickname
                    || has_mobile
                    || has_password
                {
                    // Store all headers for later use (now assigning to the outer `headers`)
                    headers = raw_headers.iter().map(|s| s.to_string()).collect();

                    email_idx = headers
                        .iter()
                        .position(|h| h == "email" || h == "mail" || h == "ODQP_LB_MAIL_CONTACT")
                        .unwrap_or(usize::MAX);
                    username_idx = headers
                        .iter()
                        .position(|h| h == "username")
                        .unwrap_or(usize::MAX);
                    password_idx = headers
                        .iter()
                        .position(|h| h == "password" || h == "hash")
                        .unwrap_or(usize::MAX);
                    phone_idx = headers
                        .iter()
                        .position(|h| {
                            h == "mobile_phone" || h == "phone" || h == "ODQP_LB_TEL_CONTACT"
                        })
                        .unwrap_or(usize::MAX);
                    name_idx = headers
                        .iter()
                        .position(|h| h == "nickname" || h == "name")
                        .unwrap_or(usize::MAX);

                    header_detected = true;
                    continue; // skip header line
                }
            }
            // -------- 2. CSV (if header was detected and not parsed yet) --------
            if header_detected && !parsed && trimmed.contains(separator) {
                log_tx.send(LogEntry::info("CSV with header detected"))?;
                let fields: Vec<&str> = trimmed.split(separator).map(|s| s.trim()).collect();
                let mut other_parts = Vec::new();
                let mut email = String::new();
                let mut username = String::new();
                let mut name = String::new();
                let mut password = String::new();

                if email_idx < fields.len() {
                    // log_tx.send(LogEntry::debug("Email detected"))?;
                    email = fields[email_idx].to_string();
                }

                if password_idx < fields.len() {
                    // log_tx.send(LogEntry::debug("Password detected"))?;
                    password = fields[password_idx].to_string();
                }

                if phone_idx < fields.len() {
                    // log_tx.send(LogEntry::debug("Phone detected"))?;
                    let phone_clean = clean_phone(fields[phone_idx]);
                    if email.is_empty() && !phone_clean.is_empty() {
                        email = phone_clean;
                    } else {
                        other_parts.push(format!("Phone: {}", phone_clean));
                    }
                }

                if username_idx < fields.len() {
                    // log_tx.send(LogEntry::debug("Username detected"))?;
                    username = fields[username_idx].to_string();
                }

                if name_idx < fields.len() {
                    // log_tx.send(LogEntry::debug("Name detected"))?;
                    name = fields[name_idx].to_string();
                }

                // Collect all other fields with their column names
                for (i, field) in fields.iter().enumerate() {
                    if i != email_idx
                        && i != phone_idx
                        && i != username_idx
                        && i != name_idx
                        && i != password_idx
                    {
                        if !field.is_empty() {
                            // Use the stored `headers` to get the column name
                            let header_name =
                                headers.get(i).map(|s| s.as_str()).unwrap_or("unknown");
                            other_parts.push(format!("{}: {}", header_name, field));
                        }
                    }
                }
                let other = other_parts.join(" | ");

                if !email.is_empty() || !username.is_empty() {
                    append_row(
                        &mut appender,
                        &email,
                        &username,
                        &password,
                        "",
                        &name,
                        &other,
                        &self.file_name,
                    )?;
                    total_rows += 1;
                    rows_since_flush += 1;
                    rows_since_commit += 1;
                    parsed = true;
                }
            }

            // -------- 3. android:// --------
            if !parsed && trimmed.starts_with("android://") {
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() >= 4 {
                    let password = parts.last().unwrap().to_string();
                    let email_or_username = parts[parts.len() - 2].to_string();
                    let url = parts[..parts.len() - 2].join(":");
                    let (email, username) = if email_or_username.contains('@') {
                        (email_or_username, String::new())
                    } else {
                        (String::new(), email_or_username)
                    };
                    append_row(
                        &mut appender,
                        &email,
                        &username,
                        &password,
                        &url,
                        "",
                        "",
                        &self.file_name,
                    )?;
                    total_rows += 1;
                    rows_since_flush += 1;
                    rows_since_commit += 1;
                    parsed = true;
                }
            }

            // -------- 4. French CSV --------
            if !parsed && trimmed.contains(',') && is_french_style(trimmed) {
                let fields: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
                let name_val = fields.get(0).unwrap_or(&"").to_string();
                let phone_raw = fields.get(4).unwrap_or(&"").trim();
                let phone_clean = clean_phone(phone_raw);
                if !phone_clean.is_empty() && !name_val.is_empty() {
                    append_row(
                        &mut appender,
                        &phone_clean,
                        "",
                        "",
                        "",
                        &name_val,
                        "",
                        &self.file_name,
                    )?;
                    total_rows += 1;
                    rows_since_flush += 1;
                    rows_since_commit += 1;
                    parsed = true;
                }
            }

            // -------- 5. Colon combos --------
            if !parsed && trimmed.contains(':') {
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() >= 3 {
                    let password = parts.last().unwrap().to_string();
                    let email_or_username = parts[parts.len() - 2].to_string();
                    let url = parts[..parts.len() - 2].join(":");
                    let (email, username) = if email_or_username.contains('@') {
                        (email_or_username, String::new())
                    } else {
                        (String::new(), email_or_username)
                    };
                    append_row(
                        &mut appender,
                        &email,
                        &username,
                        &password,
                        &url,
                        "",
                        "",
                        &self.file_name,
                    )?;
                    total_rows += 1;
                    rows_since_flush += 1;
                    rows_since_commit += 1;
                    parsed = true;
                } else if parts.len() == 2 {
                    let first = parts[0].to_string();
                    let second = parts[1].to_string();
                    let (email, username) = if first.chars().all(|c| c.is_ascii_digit())
                        && (7..=15).contains(&first.len())
                    {
                        (first, String::new())
                    } else if first.contains('@') {
                        (first, String::new())
                    } else {
                        (String::new(), first)
                    };
                    let password = second;
                    append_row(
                        &mut appender,
                        &email,
                        &username,
                        &password,
                        "",
                        "",
                        "",
                        &self.file_name,
                    )?;
                    total_rows += 1;
                    rows_since_flush += 1;
                    rows_since_commit += 1;
                    parsed = true;
                }
            }

            if !parsed {
                skipped_rows += 1;
            }

            // -------- Flush and commit logic (for non-JSON rows) --------
            if rows_since_flush >= FLUSH_INTERVAL {
                appender.flush()?;
                rows_since_flush = 0;
            }
            if rows_since_commit >= COMMIT_INTERVAL {
                appender.flush()?;
                conn.execute("COMMIT", [])?;
                conn.execute("BEGIN TRANSACTION", [])?;
                rows_since_commit = 0;
                let elapsed = start_time.elapsed().as_secs_f64();
                let rate = total_rows as f64 / elapsed;
                let _ = log_tx.send(LogEntry::info(format!(
                    "Committed {} rows, {:.0} rows/sec",
                    total_rows, rate
                )));
            }

            // Log progress every 1M rows
            if total_rows % 1_000_000 == 0 && total_rows > 0 {
                appender.flush()?;
                let elapsed = start_time.elapsed().as_secs_f64();
                let rate = total_rows as f64 / elapsed;
                let _ = log_tx.send(LogEntry::info(format!(
                    "Parsed {} rows (skipped: {}), {:.0} rows/sec",
                    total_rows, skipped_rows, rate
                )));
            }
        }

        appender.flush()?;
        conn.execute("COMMIT", [])?;

        let elapsed = start_time.elapsed().as_secs_f64();
        let _ = log_tx.send(LogEntry::success(format!(
            "Finished loading {} rows in {:.2}s ({:.0} rows/sec)",
            total_rows,
            elapsed,
            total_rows as f64 / elapsed
        )));

        Ok(format!("Finished sorting {}", self.file_name))
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
                            return format!("{}:{}:{}", split[1], split[2], split[0]);
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
