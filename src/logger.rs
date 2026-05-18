use chrono::Utc;
use csv::WriterBuilder;
use std::fs::OpenOptions;
use std::path::Path;

use crate::client::BoxError;

pub fn write_row(output: &str, values: &[(u16, u16)]) -> Result<(), BoxError> {
    let path = Path::new(output);
    let write_header = !path.exists();

    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;

    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    if write_header {
        let mut header: Vec<String> = vec!["timestamp".into()];
        for (addr, _) in values {
            header.push(addr.to_string());
        }
        wtr.write_record(&header)?;
    }

    let ts = Utc::now().to_rfc3339();
    let mut row: Vec<String> = vec![ts];
    for (_, val) in values {
        row.push(val.to_string());
    }
    wtr.write_record(&row)?;
    wtr.flush()?;
    Ok(())
}
