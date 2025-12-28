use md5::{self, Digest};
use rayon::prelude::*;
use std::fs::{self, File};
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

pub struct HashFile {
    file: File,
    hasher: md5::Md5,
    total_size: u64,
    bytes_read: u64,
    hashdb: String,
}

impl HashFile {
    pub fn new(path: &str, db_location: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let total_size = std::fs::metadata(path)?.len();
        let hashdb = db_location.clone().to_string() + "/hashdb";

        Ok(HashFile {
            file,
            hasher: md5::Md5::new(),
            total_size,
            bytes_read: 0,
            hashdb,
        })
    }
    pub fn update(&mut self) -> std::io::Result<bool> {
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

    pub fn finalize(&self) -> io::Result<String> {
        let cloned = self.hasher.clone();
        let result = cloned.finalize();
        let path = self.hashdb.clone();

        let file = File::open(&path);
        let mut exist = false;

        if let Ok(file) = file {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(l) = line
                    && l.trim() == format!("{:x}", result)
                {
                    exist = true;
                    break;
                }
            }
        }

        if !exist {
            let mut hashdb = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;

            hashdb.write_all(format!("{:x}\n", result).as_bytes())?;
        }

        Ok(format!("{:x}", result))
    }
}

pub struct Sort {
    file: File,
}

impl Sort {
    pub fn new(path: String) -> color_eyre::Result<Self> {
        let file = File::open(path)?;
        Ok(Sort { file })
    }

    pub fn update() {}

    pub fn progress() -> f64 {
        0.0
    }

    pub fn finalize() -> color_eyre::Result<String> {
        Ok("Succesfull".to_string())
    }
}
