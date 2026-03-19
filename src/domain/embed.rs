use crate::domain::png::PngChunk;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

/// draw.io XML から zTXt チャンクデータを生成する
pub fn create_ztxt_data(xml: &str) -> anyhow::Result<Vec<u8>> {
    let keyword = b"mxGraphModel";
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes())?;
    let compressed = encoder.finish()?;

    let mut data = Vec::with_capacity(keyword.len() + 1 + 1 + compressed.len());
    data.extend_from_slice(keyword);
    data.push(0); // null separator
    data.push(0); // compression method: deflate
    data.extend_from_slice(&compressed);
    Ok(data)
}

/// 既存の PNG チャンク一覧に zTXt チャンクを IEND の直前に挿入する
pub fn inject_ztxt_chunk(chunks: &mut Vec<PngChunk>, xml: &str) -> anyhow::Result<()> {
    let ztxt_data = create_ztxt_data(xml)?;
    let ztxt_chunk = PngChunk {
        chunk_type: *b"zTXt",
        data: ztxt_data,
    };
    let iend_pos = chunks
        .iter()
        .position(|c| &c.chunk_type == b"IEND")
        .ok_or_else(|| anyhow::anyhow!("IEND chunk not found"))?;
    chunks.insert(iend_pos, ztxt_chunk);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ztxt_data_starts_with_keyword() {
        let data = create_ztxt_data("<mxfile/>").unwrap();
        assert!(data.starts_with(b"mxGraphModel\0\0"));
    }

    #[test]
    fn test_inject_ztxt_chunk_before_iend() {
        let plain = std::fs::read("tests/fixtures/plain.png").unwrap();
        let mut chunks = crate::domain::png::parse_chunks(&plain).unwrap();
        let original_len = chunks.len();
        inject_ztxt_chunk(&mut chunks, "<mxfile/>").unwrap();
        assert_eq!(chunks.len(), original_len + 1);
        let ztxt = &chunks[chunks.len() - 2];
        assert_eq!(&ztxt.chunk_type, b"zTXt");
        assert_eq!(&chunks.last().unwrap().chunk_type, b"IEND");
    }

    #[test]
    fn test_inject_ztxt_chunk_no_iend() {
        let mut chunks = vec![PngChunk {
            chunk_type: *b"IHDR",
            data: vec![],
        }];
        let result = inject_ztxt_chunk(&mut chunks, "<mxfile/>");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("IEND"));
    }
}
