use std::path::PathBuf;
use duckdb::Connection;
use color_eyre::Result;

pub struct Searcher {
    db_path: PathBuf,
    email_to_search: String,
    output: Vec<String>,          // now stores formatted lines
}

impl Searcher {
    pub fn new(
        db_dir: String,
        _export_dir: String,      // kept for compatibility, unused
        email_to_search: String,
    ) -> Result<Self, color_eyre::Report> {
        let db_path = std::env::current_dir()
            .unwrap()
            .join(db_dir)
            .join("sorted")
            .join("data.db");

        if !db_path.exists() {
            return Err(color_eyre::eyre::eyre!(
                "database file not found at {:?}",
                db_path
            ));
        }

        Ok(Searcher {
            db_path,
            email_to_search,
            output: Vec::new(),
        })
    }

    pub fn search(&mut self) -> Result<bool, color_eyre::Report> {
        self.output.clear();

        // Open the persistent database file (not in‑memory)
        let conn = Connection::open(&self.db_path)?;
        conn.execute("PRAGMA memory_limit='8GB';", [])?;
        conn.execute("PRAGMA temp_directory='/tmp/duckdb_temp';", [])?;
        conn.execute("PRAGMA threads=2;", [])?;


        let max_results = 10_000; // hard cap, tune as needed
        // Query the 'contacts' table directly – no need for read_parquet
        let mut stmt = conn.prepare(&format!(
            "SELECT email, password, url, username, name, other, origin
             FROM contacts
             WHERE email LIKE ?1 || '%'
            LIMIT {}",
        max_results
    ))?;

        // Bind only the email search term (one parameter)
        let mut rows = stmt.query([&self.email_to_search])?;
        let mut count = 0;
        while let Some(row) = rows.next()? {
            let email: Option<String> = row.get(0)?;
            let password: Option<String> = row.get(1)?;
            let url: Option<String> = row.get(2)?;
            let username: Option<String> = row.get(3)?;
            let name: Option<String> = row.get(4)?;
            let other: Option<String> = row.get(5)?;
            let origin: Option<String> = row.get(6)?;

            let mut parts = Vec::new();


            // Only add email if it's not empty
            if let Some(email) = email {
                if !email.is_empty() {
                    parts.push(format!("email : {}", email));
                }
            }

            // Only add password if not empty
            if let Some(password) = password {
                if !password.is_empty() {
                    parts.push(format!("password : {}", password));
                }
            }

            // Add other fields without labels (or with labels if you prefer)
            if let Some(url) = url {
                if !url.is_empty() {
                    parts.push(format!("url : {}",url));
                }
            }
            if let Some(username) = username {
                if !username.is_empty() {
                    parts.push(format!("username : {}",username));
                }
            }
            if let Some(name) = name {
                if !name.is_empty() {
                    parts.push(format!("name : {}",name));
                }
            }
            if let Some(other) = other {
                if !other.is_empty() {
                    parts.push(format!("other : {}",other));
                }
            }

            if let Some(origin) = origin {
                if !origin.is_empty() {
                    parts.push(format!("origin : {}",origin));
                }
            }

            // Join all non‑empty parts with " | "
            let line = parts.join(" | ");
            self.output.push(line);
            count += 1;
            if count >= max_results {
                break;
            }
        }

        Ok(true)
    }

    pub fn update(&mut self) {
        // no‑op
    }

    pub fn progress(&self) -> f64 {
        1.0   // query is atomic – considered complete after search()
    }

    pub fn get_results(&self) -> &[String] {
        &self.output
    }
}
