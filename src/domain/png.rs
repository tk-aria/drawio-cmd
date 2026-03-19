/// PNG チャンクの構造体
pub struct PngChunk {
    pub chunk_type: [u8; 4],
    pub data: Vec<u8>,
}

const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// PNG シグネチャを検証する
pub fn validate_signature(data: &[u8]) -> anyhow::Result<()> {
    if data.len() < 8 || data[..8] != PNG_SIGNATURE {
        anyhow::bail!("Invalid PNG signature");
    }
    Ok(())
}

/// PNG バイナリからチャンク一覧をパースする
pub fn parse_chunks(data: &[u8]) -> anyhow::Result<Vec<PngChunk>> {
    validate_signature(data)?;
    let mut chunks = Vec::new();
    let mut offset = 8; // skip signature

    while offset < data.len() {
        if offset + 8 > data.len() {
            anyhow::bail!(
                "Truncated PNG: not enough data for chunk header at offset {}",
                offset
            );
        }

        let length = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        let chunk_type: [u8; 4] = [
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ];

        let data_start = offset + 8;
        let data_end = data_start + length;
        if data_end + 4 > data.len() {
            anyhow::bail!(
                "Truncated PNG: chunk '{}' at offset {} declares length {} but data is insufficient",
                String::from_utf8_lossy(&chunk_type),
                offset,
                length
            );
        }

        let chunk_data = data[data_start..data_end].to_vec();
        // skip CRC (4 bytes)
        chunks.push(PngChunk {
            chunk_type,
            data: chunk_data,
        });

        offset = data_end + 4; // length(4) + type(4) + data + crc(4), we started at length
    }

    Ok(chunks)
}

/// チャンク一覧から PNG バイナリを再構築する
pub fn build_png(chunks: &[PngChunk]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&PNG_SIGNATURE);

    for chunk in chunks {
        let length = chunk.data.len() as u32;
        buf.extend_from_slice(&length.to_be_bytes());
        buf.extend_from_slice(&chunk.chunk_type);
        buf.extend_from_slice(&chunk.data);
        let crc = calc_crc(&chunk.chunk_type, &chunk.data);
        buf.extend_from_slice(&crc.to_be_bytes());
    }

    buf
}

/// CRC32 を計算する (type + data に対して)
pub fn calc_crc(chunk_type: &[u8; 4], data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(chunk_type);
    hasher.update(data);
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_signature_valid() {
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert!(validate_signature(&data).is_ok());
    }

    #[test]
    fn test_validate_signature_invalid() {
        let data = [0x00; 8];
        assert!(validate_signature(&data).is_err());
    }

    #[test]
    fn test_validate_signature_too_short() {
        let data = [0x89, 0x50];
        assert!(validate_signature(&data).is_err());
    }

    #[test]
    fn test_parse_chunks_returns_ihdr_and_iend() {
        let png = std::fs::read("tests/fixtures/plain.png").unwrap();
        let chunks = parse_chunks(&png).unwrap();
        assert_eq!(&chunks.first().unwrap().chunk_type, b"IHDR");
        assert_eq!(&chunks.last().unwrap().chunk_type, b"IEND");
    }

    #[test]
    fn test_build_png_roundtrip() {
        let original = std::fs::read("tests/fixtures/plain.png").unwrap();
        let chunks = parse_chunks(&original).unwrap();
        let rebuilt = build_png(&chunks);
        assert_eq!(original, rebuilt);
    }

    #[test]
    fn test_calc_crc_known_value() {
        let crc = calc_crc(b"IEND", &[]);
        assert_eq!(crc, 0xAE426082);
    }

    #[test]
    fn test_parse_chunks_empty_after_signature() {
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let result = parse_chunks(&data);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_chunks_truncated_chunk() {
        let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0A]); // length=10
        data.extend_from_slice(b"IHDR"); // type
                                         // data が足りない → エラー
        assert!(parse_chunks(&data).is_err());
    }
}
