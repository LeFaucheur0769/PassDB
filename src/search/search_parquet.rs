use anyhow::Result;
use duckdb::Connection;

pub fn search_by_email(
    email: &str,
) -> Result<
    Vec<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
> {
    let conn = Connection::open_in_memory()?;
    let mut stmt = conn.prepare(
        "SELECT email, username ,password, url, name, other
FROM read_parquet('output/sorted/database.parquet')
WHERE email = ?1",
    )?;
    let mut rows = stmt.query([email])?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        results.push((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
        ));
    }

    Ok(results)
}


