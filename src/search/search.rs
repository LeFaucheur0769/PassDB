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
            .join("database.parquet");

        if !db_path.exists() {
            return Err(color_eyre::eyre::eyre!(
                "Parquet file not found at {:?}",
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

        let conn = Connection::open_in_memory()?;

        // Use substring match (LIKE) to mimic the old .contains() behaviour
        let mut stmt = conn.prepare(
            "SELECT email, username, password, url, name, other
             FROM read_parquet(?1)
             WHERE email LIKE '%' || ?2 || '%'"
        )?;

        let db_path_str = self.db_path.to_string_lossy().to_string();
        let mut rows = stmt.query([db_path_str, self.email_to_search.clone()])?;

        while let Some(row) = rows.next()? {
            let email: Option<String> = row.get(0)?;
            let username: Option<String> = row.get(1)?;
            let password: Option<String> = row.get(2)?;
            let url: Option<String> = row.get(3)?;
            let name: Option<String> = row.get(4)?;
            let other: Option<String> = row.get(5)?;

            // Format the row as a tab‑separated line (replace with your preferred delimiter)
            let line = format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                email.unwrap_or_default(),
                username.unwrap_or_default(),
                password.unwrap_or_default(),
                url.unwrap_or_default(),
                name.unwrap_or_default(),
                other.unwrap_or_default()
            );
            self.output.push(line);
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
