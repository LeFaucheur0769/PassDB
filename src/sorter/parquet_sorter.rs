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

fn parquet_sorter() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn add_combo(contact: Contact) -> color_eyre::Result<()> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("email".to_string(), DataType::Utf8, true),
        Field::new("username".to_string(), DataType::Utf8, true),
        Field::new("password".to_string(), DataType::Utf8, true),
        Field::new("url".to_string(), DataType::Utf8, true),
        Field::new("name".to_string(), DataType::Utf8, true),
        Field::new("other".to_string(), DataType::Utf8, true),
    ]));

    let email: Option<&str> = contact.email.as_deref();
    let username: Option<&str> = contact.username.as_deref();
    let password: Option<&str> = contact.password.as_deref();
    let url: Option<&str> = contact.url.as_deref();
    let name: Option<&str> = contact.name.as_deref();
    let other: Option<&str> = contact.other.as_deref();

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(vec![email])),
            Arc::new(StringArray::from(vec![username])),
            Arc::new(StringArray::from(vec![password])),
            Arc::new(StringArray::from(vec![url])),
            Arc::new(StringArray::from(vec![name])),
            Arc::new(StringArray::from(vec![other])),
        ],
    )?;

    let file = File::create("output/sorted/database.parquet")?;
    let mut writer = ArrowWriter::try_new(file, schema, None)?;
    writer.write(&batch)?;

    Ok(())
}
