use crate::domain::{extract, png};
use std::fs;

pub fn extract_xml_from_png(input_path: &str) -> anyhow::Result<String> {
    let data = fs::read(input_path)?;
    png::validate_signature(&data)?;
    let chunks = png::parse_chunks(&data)?;
    extract::extract_drawio_xml(&chunks)
}
