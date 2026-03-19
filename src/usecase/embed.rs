use crate::domain::{embed, png};
use std::fs;

pub fn embed_xml_into_png(xml_path: &str, png_path: &str) -> anyhow::Result<Vec<u8>> {
    let xml = fs::read_to_string(xml_path)?;
    let png_data = fs::read(png_path)?;
    png::validate_signature(&png_data)?;
    let mut chunks = png::parse_chunks(&png_data)?;
    embed::inject_ztxt_chunk(&mut chunks, &xml)?;
    Ok(png::build_png(&chunks))
}
