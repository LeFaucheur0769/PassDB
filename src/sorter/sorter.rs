use clap::builder::Str;
use crossterm::event::read;
use dashmap::DashMap;
use md5::{self, Digest};
use ratatui::symbols::line;
use rayon::{prelude::*, vec};
use std::alloc::System;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::result::Result::Ok;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{clone, io};

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

    pub fn sort_optimised(&mut self) -> color_eyre::Result<String> {
        // Define some settings that can be tweaked to modify the process
        let nrb_lines: u64 = 1000000; // By default, reads one milion lines by one milion to
        // optimise the ram usage

        // First make sure that the output_dir exists
        fs::create_dir_all(&self.output_dir)?;

        // Create a concurent hashmap to group lines by their first characters. By using dashmap i
        // can read/write simultaniously without locking the entire hashmap. Mutex make sure that
        // only one thread can modify the vec at a time.
        let groups: DashMap<String, Arc<Mutex<Vec<String>>>> = DashMap::new();

        // Read the lines from the file
        let mut reader = BufReader::new(&self.file);
        let mut line = String::new();
        let mut batch = Vec::new();
        loop {
            line.clear();

            let bytes_read = reader.read_line(&mut line)?;

            if bytes_read == 0 {
                // EOF reached
                if !batch.is_empty() {
                    //    println!("Processing final batch of {} lines", batch.len()); // Used for debugging // This will
                    // actually print nothing due to ratatui
                }
                break;
            }
            // Add the line to batch ( we need to clone it because we will reuse the buffer)
            batch.push(line.clone());

            // If batch is full : process it
            if batch.len() >= nrb_lines as usize {
                //println!("Processing batch of {} lines", batch.len()); // Used for debugging

                // Start of the sorting process for the lines contained in the batch

                batch
                    // Using iter allows to process lines in parallel using rayon
                    .par_iter()
                    // Process only the non empty lines
                    .filter(|line| !line.trim().is_empty())
                    // For earch non empty line, process it in parallel
                    .for_each(|line| {
                        // trim whitespaces and convert the line to String
                        let trimmed = line.trim().to_string();
                        let first_3 = trimmed.chars().take(3).collect::<String>();
                        if !first_3.is_empty() {
                            groups
                                .entry(first_3)
                                .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
                                .lock()
                                .unwrap()
                                .push(trimmed);
                        }
                    });
                // Write each group to its own file, also in parallel
                groups.par_iter().try_for_each(|entry| {
                    let (file_name, lines_arc) = entry.pair();
                    let file_path =
                        PathBuf::from(&self.output_dir).join(format!("{}.txt", file_name));
                    let lines = Arc::try_unwrap(lines_arc.clone())
                        .map_err(|_| {
                            color_eyre::eyre::eyre!(
                                "Failed to unwrap Arc - multiple references still exist"
                            )
                        })?
                        .into_inner()
                        .map_err(|_| {
                            color_eyre::eyre::eyre!(
                                "Mutex poison error - a thread panicked while holding the lock"
                            )
                        })?;
                    let content = lines.join("\n") + "\n";

                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(file_path)?;
                    file.write_all(content.as_bytes())?;
                    Ok::<(), color_eyre::Report>(())
                })?;
                batch.clear();
            }
        }
        exit(11);

        Ok(format!(
            "Finished sorting {} (created {} sorted files)",
            self.file_name,
            groups.len()
        ))
    }

    pub fn sort(&mut self) -> color_eyre::Result<String> {
        let reader = BufReader::new(&self.file);
        let mut vec_first_3 = vec![];
        for (i, line) in reader.lines().enumerate() {
            let line_unwrap = match line {
                Ok(l) => l.trim().to_string(),
                Err(_) => continue, // skip invalid UTF-8 lines
            };

            let mut file_name = line_unwrap.chars().take(3).collect::<String>();

            if file_name.is_empty() {
                continue;
            }

            file_name.push_str(".txt");
            vec_first_3.push(file_name.clone());

            let mut file_path = std::path::PathBuf::from(&self.output_dir);
            file_path.push(&file_name); // append the file name

            // Open the file corresponding to the line

            let mut file_sorted_line = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?;
            writeln!(file_sorted_line, "{}", line_unwrap)?;
        }
        Ok(format!("Finished sorting {}", self.file_name))
    }

    // pub fn progress() -> f64 {
    // 0.0
    // }

    // pub fn finalize(&mut self) -> color_eyre::Result<String> {
    // Ok("Succesfull".to_string())
    // }
}
