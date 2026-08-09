use rusqlite::{Connection, Result, params};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use bloom::{BloomFilter, ASMS};
use serde::{Serialize, Deserialize};
use anyhow::Result as AnyResult; // Alias to avoid conflict with rusqlite::Result

// ------------------- Database Setup -------------------

fn setup_database(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Enable WAL mode for better concurrent read performance
    conn.pragma_update(None, "journal_mode", "WAL")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS combolist (
            id INTEGER PRIMARY KEY,
            email TEXT NOT NULL,
            password TEXT NOT NULL,
            source TEXT,
            ingested_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create an index on email for fast lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email ON combolist(email)",
        [],
    )?;

    Ok(conn)
}

// ------------------- Bloom Filter Wrapper (with serialization) -------------------

#[derive(Serialize, Deserialize)]
pub struct CombolistBloomFilter {
    filter: BloomFilter,
}

impl CombolistBloomFilter {
    // Create a new filter with a target false positive rate and expected number of items
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let filter = BloomFilter::with_rate(false_positive_rate, expected_items);
        CombolistBloomFilter { filter }
    }

    // Insert an email into the filter
    pub fn insert(&mut self, email: &str) {
        self.filter.insert(email);
    }

    // Check if an email *might* be in the set
    pub fn might_contain(&self, email: &str) -> bool {
        self.filter.contains(email)
    }

    // Serialize the filter to a byte vector (for persistence)
    pub fn to_bytes(&self) -> AnyResult<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    // Deserialize the filter from a byte slice
    pub fn from_bytes(data: &[u8]) -> AnyResult<Self> {
        Ok(bincode::deserialize(data)?)
    }
}

// ------------------- Persistence Helpers -------------------

fn save_bloom_filter(bloom: &CombolistBloomFilter, path: &str) -> AnyResult<()> {
    let data = bloom.to_bytes()?;
    let mut file = File::create(path)?;
    file.write_all(&data)?;
    Ok(())
}

fn load_bloom_filter(path: &str) -> AnyResult<CombolistBloomFilter> {
    let data = std::fs::read(path)?;
    let bloom = CombolistBloomFilter::from_bytes(&data)?;
    Ok(bloom)
}

// ------------------- Ingestion -------------------

// IMPORTANT: This function stores passwords in PLAINTEXT as you requested.
// SQLite column is now "password" (not "password_hash").
fn ingest_combolist(conn: &Connection, bloom: &mut CombolistBloomFilter, file_path: &str) -> Result<()> {
    let file = File::open(file_path).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, "file", Box::new(e)))?;
    let reader = BufReader::new(file);

    // Begin a transaction for performance
    let tx = conn.transaction()?;

    // Prepare the insert statement (now stores plaintext password)
    let mut stmt = tx.prepare(
        "INSERT INTO combolist (email, password, source) VALUES (?, ?, ?)"
    )?;

    for line in reader.lines() {
        let line = line.map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, "line", Box::new(e)))?;
        // Parse the line (assuming format: email:password)
        if let Some((email, password)) = line.split_once(':') {
            let email = email.trim();
            let password = password.trim();

            // Insert into SQLite (plaintext password)
            stmt.execute(params![email, password, file_path])?;

            // Insert into Bloom filter
            bloom.insert(email);
        }
    }

    tx.commit()?;
    Ok(())
}

// ------------------- Lookup (Two-Step) -------------------

fn lookup_email(conn: &Connection, bloom: &CombolistBloomFilter, email: &str) -> Result<Option<String>> {
    // Step 1: Check Bloom filter (in RAM, super fast)
    if !bloom.might_contain(email) {
        return Ok(None); // Definitely not present
    }

    // Step 2: Confirm with SQLite (disk, exact)
    let mut stmt = conn.prepare("SELECT password FROM combolist WHERE email = ?")?;
    let mut rows = stmt.query(params![email])?;

    if let Some(row) = rows.next()? {
        let password: String = row.get(0)?;
        Ok(Some(password))
    } else {
        // This was a false positive from the Bloom filter
        Ok(None)
    }
}

// ------------------- Main (Example Workflow) -------------------

fn sql_sorter() -> AnyResult<()> {
    let db_path = "combolist.db";
    let bloom_path = "combolist.bloom";

    // Set up the database
    let conn = setup_database(db_path)?;

    // Try to load existing Bloom filter, or create a new one
    let mut bloom = match load_bloom_filter(bloom_path) {
        Ok(filter) => {
            println!("✅ Loaded existing Bloom filter from disk.");
            filter
        }
        Err(_) => {
            println!("🔨 No existing Bloom filter found. Creating a new one...");
            // Estimate: 1 million items, 1% false positive rate
            CombolistBloomFilter::new(1_000_000, 0.01)
        }
    };

    // ------------------- INGEST DATA -------------------
    // Uncomment the line below and point to your combolist file to ingest.
    // The file should have one "email:password" per line.
    //
    // ingest_combolist(&conn, &mut bloom, "data/my_combolist.txt")?;
    //
    // After ingestion, save the Bloom filter to disk so we don't rebuild it next time.
    // save_bloom_filter(&bloom, bloom_path)?;
    // println!("💾 Bloom filter saved to disk.");

    // ------------------- EXAMPLE LOOKUP -------------------
    let test_email = "testuser@example.com";
    match lookup_email(&conn, &bloom, test_email)? {
        Some(password) => println!("🔍 Found '{}' -> password: {}", test_email, password),
        None => println!("🔍 '{}' not found in the combolist.", test_email),
    }

    Ok(())
}