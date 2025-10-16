use md5::{self, Digest};
use std::fs::File;
use std::io;
use std::io::Read;

pub fn sorter() -> std::io::Result<String> {
    let hash = hash_file("/home/grimreaper/Desktop/steam.desktop")?;
    println!("{}", hash);
    Ok(hash)
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
