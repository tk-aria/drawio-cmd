use crate::domain::render::DiagramRenderer;
use crate::domain::{embed, png};

pub fn export_drawio_to_png(
    renderer: &dyn DiagramRenderer,
    xml: &str,
    scale: f64,
) -> anyhow::Result<Vec<u8>> {
    // 1. Render XML to PNG
    let png_data = renderer.render_to_png(xml, scale)?;

    // 2. Embed original XML into the rendered PNG
    png::validate_signature(&png_data)?;
    let mut chunks = png::parse_chunks(&png_data)?;
    embed::inject_ztxt_chunk(&mut chunks, xml)?;
    Ok(png::build_png(&chunks))
}
