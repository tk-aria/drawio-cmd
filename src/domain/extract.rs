use crate::domain::png::PngChunk;
use flate2::read::ZlibDecoder;
use std::io::Read;

/// zTXt チャンクからテキストを抽出する
pub fn decode_ztxt(data: &[u8]) -> anyhow::Result<(String, String)> {
    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid zTXt: no null separator"))?;
    let keyword = String::from_utf8(data[..null_pos].to_vec())?;
    let _compression_method = data[null_pos + 1]; // 0 = deflate
    let compressed = &data[null_pos + 2..];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut text = String::new();
    decoder.read_to_string(&mut text)?;
    Ok((keyword, text))
}

/// tEXt チャンクからテキストを抽出する
pub fn decode_text(data: &[u8]) -> anyhow::Result<(String, String)> {
    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid tEXt: no null separator"))?;
    let keyword = String::from_utf8(data[..null_pos].to_vec())?;
    let text = String::from_utf8(data[null_pos + 1..].to_vec())?;
    Ok((keyword, text))
}

/// iTXt チャンクからテキストを抽出する
pub fn decode_itxt(data: &[u8]) -> anyhow::Result<(String, String)> {
    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid iTXt: no null separator"))?;
    let keyword = String::from_utf8(data[..null_pos].to_vec())?;

    let mut pos = null_pos + 1;
    if pos >= data.len() {
        anyhow::bail!("Invalid iTXt: truncated after keyword");
    }
    let compression_flag = data[pos];
    pos += 1; // compression_method
    pos += 1;

    // skip language tag (null-terminated)
    let lang_end = data[pos..]
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid iTXt: no language null terminator"))?;
    pos += lang_end + 1;

    // skip translated keyword (null-terminated)
    let trans_end = data[pos..]
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid iTXt: no translated keyword null terminator"))?;
    pos += trans_end + 1;

    let text = if compression_flag != 0 {
        let mut decoder = ZlibDecoder::new(&data[pos..]);
        let mut s = String::new();
        decoder.read_to_string(&mut s)?;
        s
    } else {
        String::from_utf8(data[pos..].to_vec())?
    };

    Ok((keyword, text))
}

/// 抽出テキストのエンコーディングを自動判定してデコードする
pub fn decode_payload(text: &str) -> anyhow::Result<String> {
    if text.starts_with('<') {
        return Ok(text.to_string());
    }
    if text.contains("%3C") || text.contains("%3E") {
        let decoded = percent_encoding::percent_decode_str(text).decode_utf8()?;
        return Ok(decoded.into_owned());
    }
    if text.starts_with("PD") || text.starts_with("eN") {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD.decode(text)?;
        return Ok(String::from_utf8(bytes)?);
    }
    Ok(text.to_string())
}

/// PNG チャンク一覧から draw.io XML を検索・抽出する
pub fn extract_drawio_xml(chunks: &[PngChunk]) -> anyhow::Result<String> {
    for chunk in chunks {
        let type_str = std::str::from_utf8(&chunk.chunk_type)?;
        let result = match type_str {
            "zTXt" => decode_ztxt(&chunk.data),
            "tEXt" => decode_text(&chunk.data),
            "iTXt" => decode_itxt(&chunk.data),
            _ => continue,
        };
        if let Ok((keyword, text)) = result {
            if keyword == "mxGraphModel" || keyword == "mxfile" {
                return decode_payload(&text);
            }
        }
    }
    anyhow::bail!(
        "No draw.io XML found in PNG. The PNG must be exported with 'Include a copy of my diagram' enabled."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_payload_raw_xml() {
        let xml = "<mxfile></mxfile>";
        assert_eq!(decode_payload(xml).unwrap(), xml);
    }

    #[test]
    fn test_decode_payload_url_encoded() {
        let encoded = "%3Cmxfile%3E%3C%2Fmxfile%3E";
        assert_eq!(decode_payload(encoded).unwrap(), "<mxfile></mxfile>");
    }

    #[test]
    fn test_decode_payload_base64_pd_prefix() {
        use base64::Engine;
        let xml = "<?xml version=\"1.0\"?><mxfile></mxfile>";
        let b64 = base64::engine::general_purpose::STANDARD.encode(xml);
        assert!(b64.starts_with("PD"));
        assert_eq!(decode_payload(&b64).unwrap(), xml);
    }

    #[test]
    fn test_decode_payload_passthrough() {
        // Payload that doesn't match any encoding heuristic is returned as-is
        let text = "some random text";
        assert_eq!(decode_payload(text).unwrap(), text);
    }

    #[test]
    fn test_extract_drawio_xml_from_embedded_png() {
        let data = std::fs::read("tests/fixtures/simple_embedded.png").unwrap();
        let chunks = crate::domain::png::parse_chunks(&data).unwrap();
        let xml = extract_drawio_xml(&chunks).unwrap();
        assert!(xml.contains("<mxfile"));
        assert!(xml.contains("mxGraphModel"));
    }

    #[test]
    fn test_extract_drawio_xml_no_ztxt() {
        let plain = std::fs::read("tests/fixtures/plain.png").unwrap();
        let chunks = crate::domain::png::parse_chunks(&plain).unwrap();
        let result = extract_drawio_xml(&chunks);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No draw.io XML found"));
    }

    #[test]
    fn test_decode_ztxt_no_null_separator() {
        let data = b"mxGraphModel";
        assert!(decode_ztxt(data).is_err());
    }

    #[test]
    fn test_decode_ztxt_invalid_compressed_data() {
        let mut data = b"mxGraphModel\0\0".to_vec();
        data.extend_from_slice(&[0xFF, 0xFE, 0xFD]);
        assert!(decode_ztxt(&data).is_err());
    }
}
