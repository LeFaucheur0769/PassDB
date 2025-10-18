use clap::builder::Str;
use crossterm::cursor::MoveToNextLine;
use md5::{self, Digest};
use ratatui::buffer;
use std::fs::{self, File, metadata};
use std::io;
use std::io::Read;
use std::path::PathBuf;

pub fn sorter() -> std::io::Result<String> {
    let hash = hash_file("/home/grimreaper/Desktop/steam.desktop")?;
    println!("{}", hash);
    Ok(hash)
}

pub struct HashFile {
    file: File,
    hasher: md5::Md5,
    total_size: u64,
    bytes_read: u64,
}

impl HashFile {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let total_size = std::fs::metadata(path)?.len();

        Ok(HashFile {
            file: File::open(path)?,
            hasher: md5::Md5::new(),
            total_size,
            bytes_read: 0,
        })
    }
    pub fn update(&mut self) -> std::io::Result<bool> {
        let mut buffer = [0u8; 8192];
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

    pub fn finalize(&self) -> String {
        let cloned = self.hasher.clone();
        let result = cloned.finalize();
        format!("{:x}", result)
    }
}

fn hash_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = md5::Md5::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
