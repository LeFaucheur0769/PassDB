use anyhow::Result;
use arrow::{
    array::StringArray,
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use parquet::arrow::ArrowWriter;
use std::{fs::File, sync::Arc};

#[derive(Debug, Clone)]
pub struct Contact {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub name: Option<String>,
    pub other: Option<String>,
}

pub fn init_database(db_path: &str) -> anyhow::Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS contacts (
            email VARCHAR,
            username VARCHAR,
            password VARCHAR,
            url VARCHAR,
            name VARCHAR,
            other VARCHAR,
            origin VARCHAR
        )"
    )?;
    Ok(conn)
}

use duckdb::{Connection};
use std::path::Path;

pub fn add_combo(contacts: &[Contact]) ->  anyhow::Result<()> {
    if contacts.is_empty() {
        return Ok(());
    }

    let path = "output/sorted/database.parquet";
    let conn = Connection::open_in_memory()?;

    // 1. Load existing data (if any)
    if Path::new(path).exists() {
        conn.execute_batch(&format!(
            "CREATE OR REPLACE TABLE existing AS SELECT * FROM read_parquet('{}')",
            path
        ))?;
    } else {
        conn.execute_batch(
            "CREATE TABLE existing (
                email VARCHAR,
                username VARCHAR,
                password VARCHAR,
                url VARCHAR,
                name VARCHAR,
                other VARCHAR
            )",
        )?;
    }

    // 2. Insert all new contacts in one go using a prepared statement
    let mut stmt = conn.prepare(
        "INSERT INTO existing VALUES (?, ?, ?, ?, ?, ?)"
    )?;

    for contact in contacts {
        stmt.execute([
            &contact.email,
            &contact.username,
            &contact.password,
            &contact.url,
            &contact.name,
            &contact.other,
        ])?;
    }

    // 3. Write the whole table back (overwrites the file)
    conn.execute_batch(&format!(
        "COPY existing TO '{}' (FORMAT PARQUET)",
        path
    ))?;

    Ok(())
}

pub fn export_to_parquet(conn: &Connection, parquet_path: &str) -> anyhow::Result<()> {
    conn.execute_batch(&format!(
        "COPY contacts TO '{}' (FORMAT PARQUET)",
        parquet_path
    ))?;
    Ok(())
}
