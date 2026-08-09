use std::fs::File;
use std::sync::Mutex;

pub static LOG_FILE: std::sync::OnceLock<Mutex<File>> = std::sync::OnceLock::new();

pub fn init_log(path: &str) {
    let file = File::create(path).expect("Could not create log file");
    LOG_FILE.set(Mutex::new(file)).ok();
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if let Some(f) = crate::tools::logs::LOG_FILE.get() {
            use std::io::Write;
            let mut f: std::sync::MutexGuard<std::fs::File> = f.lock().unwrap();
            writeln!(f, $($arg)*).ok();
        }
    };
}