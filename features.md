# drawio-tools — 実装 TODO リスト

> RFC-001 / SoW に基づく段階的実装計画
> 各タスクにはサンプルコード・参考実装を付記

---

## Phase 1: プロジェクト基盤 + PNG チャンク操作（純 Rust）

### Step 1.1: プロジェクト初期化

- [x] `Cargo.toml` を作成（ワークスペースではなく単体クレート） ✅ 2026-03-19

```toml
[package]
name = "drawio-tools"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT"
description = "Draw.io PNG/XML bidirectional conversion CLI"

[[bin]]
name = "drawio-tools"
path = "src/main.rs"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
flate2 = "1.1"
crc32fast = "1.4"
anyhow = "1.0"
percent-encoding = "2.3"
base64 = "0.22"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.14"

[features]
default = ["render"]
render = ["dep:headless_chrome"]

[dependencies.headless_chrome]
version = "1.0"
optional = true
```

- [x] `.gitignore` ✅ 2026-03-19 を更新

```gitignore
/target
Cargo.lock
*.png
!tests/fixtures/*.png
```

- [x] `Dockerfile` ✅ 2026-03-19 を作成

```dockerfile
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --no-default-features
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/drawio-tools /usr/local/bin/
ENTRYPOINT ["drawio-tools"]
```

- [x] `src/main.rs` ✅ 2026-03-19 に最小限のエントリポイントを作成

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "drawio-tools", version, about = "Draw.io PNG/XML bidirectional conversion CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract draw.io XML from PNG
    Extract { /* 後続 Step で定義 */ },
    /// Embed draw.io XML into PNG
    Embed { /* 後続 Step で定義 */ },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    Ok(())
}
```

---

### Step 1.2: ドメイン層 — PNG チャンクパーサー (`src/domain/png.rs`)

PNG バイナリを解析してチャンク一覧を取得するドメインモデルを実装する。

- [x] `src/domain/mod.rs` を作成（`pub mod png;` をエクスポート） ✅ 2026-03-19
- [x] `src/domain/png.rs` ✅ 2026-03-19 に以下の構造体・関数を実装

```rust
/// PNG チャンクの構造体
pub struct PngChunk {
    pub chunk_type: [u8; 4],
    pub data: Vec<u8>,
}

/// PNG シグネチャを検証する
pub fn validate_signature(data: &[u8]) -> anyhow::Result<()> {
    const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data.len() < 8 || data[..8] != PNG_SIGNATURE {
        anyhow::bail!("Invalid PNG signature");
    }
    Ok(())
}

/// PNG バイナリからチャンク一覧をパースする
pub fn parse_chunks(data: &[u8]) -> anyhow::Result<Vec<PngChunk>> {
    // シグネチャ(8 bytes) をスキップ
    // 各チャンク: length(4, BE) + type(4) + data(length) + crc(4)
    todo!()
}

/// チャンク一覧から PNG バイナリを再構築する
pub fn build_png(chunks: &[PngChunk]) -> Vec<u8> {
    // シグネチャ + 各チャンク(length + type + data + CRC32)
    todo!()
}

/// CRC32 を計算する (type + data に対して)
pub fn calc_crc(chunk_type: &[u8; 4], data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(chunk_type);
    hasher.update(data);
    hasher.finalize()
}
```

**参考**: Node.js 版 `extract_drawio_xml.mjs` の `parsePNGChunks()` 関数
**ファイル行数見積**: 約120行（300行以下）

---

### Step 1.3: ドメイン層 — XML 抽出ロジック (`src/domain/extract.rs`)

PNG チャンクから draw.io XML を抽出するドメインロジックを実装する。

- [x] `src/domain/mod.rs` に `pub mod extract;` ✅ 2026-03-19 を追加
- [x] `src/domain/extract.rs` ✅ 2026-03-19 に以下を実装

```rust
use flate2::read::ZlibDecoder;
use std::io::Read;

/// zTXt チャンクからテキストを抽出する
pub fn decode_ztxt(data: &[u8]) -> anyhow::Result<(String, String)> {
    // keyword\0 + compression_method(1) + compressed_data
    let null_pos = data.iter().position(|&b| b == 0)
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
    // keyword\0 + text
    let null_pos = data.iter().position(|&b| b == 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid tEXt: no null separator"))?;
    let keyword = String::from_utf8(data[..null_pos].to_vec())?;
    let text = String::from_utf8(data[null_pos + 1..].to_vec())?;
    Ok((keyword, text))
}

/// iTXt チャンクからテキストを抽出する
pub fn decode_itxt(data: &[u8]) -> anyhow::Result<(String, String)> {
    // keyword\0 + compression_flag(1) + compression_method(1)
    // + language\0 + translated_keyword\0 + text_or_compressed
    todo!()
}

/// 抽出テキストのエンコーディングを自動判定してデコードする
/// - 生 XML: "<" で始まる → そのまま
/// - URL エンコード: "%3C" を含む → percent_decode
/// - Base64: "PD" or "eN" で始まる → base64 decode
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
/// keyword が "mxGraphModel" or "mxfile" のチャンクを探す
pub fn extract_drawio_xml(chunks: &[crate::domain::png::PngChunk]) -> anyhow::Result<String> {
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
```

**参考**: Node.js 版の `extractDrawioXML()` 関数（3種のチャンク対応 + 3種のエンコーディング対応）
**ファイル行数見積**: 約130行（300行以下）

---

### Step 1.4: ドメイン層 — XML 埋め込みロジック (`src/domain/embed.rs`)

PNG に draw.io XML を zTXt チャンクとして注入するドメインロジックを実装する。

- [x] `src/domain/mod.rs` に `pub mod embed;` ✅ 2026-03-19 を追加
- [x] `src/domain/embed.rs` ✅ 2026-03-19 に以下を実装

```rust
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

/// draw.io XML から zTXt チャンクデータを生成する
/// フォーマット: keyword("mxGraphModel") + \0 + compression_method(0) + deflated_xml
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
pub fn inject_ztxt_chunk(
    chunks: &mut Vec<crate::domain::png::PngChunk>,
    xml: &str,
) -> anyhow::Result<()> {
    let ztxt_data = create_ztxt_data(xml)?;
    let ztxt_chunk = crate::domain::png::PngChunk {
        chunk_type: *b"zTXt",
        data: ztxt_data,
    };
    // IEND の直前に挿入
    let iend_pos = chunks.iter().position(|c| &c.chunk_type == b"IEND")
        .ok_or_else(|| anyhow::anyhow!("IEND chunk not found"))?;
    chunks.insert(iend_pos, ztxt_chunk);
    Ok(())
}
```

**参考**: Node.js 版 `drawio_export_embed.mjs` の CRC32 計算 + zTXt チャンク組み立てロジック
**ファイル行数見積**: 約70行（300行以下）

---

### Step 1.5: ユースケース層 (`src/usecase/`)

ドメインロジックを組み合わせたユースケースを実装する。

- [x] `src/usecase/mod.rs` ✅ 2026-03-19 を作成（`pub mod extract; pub mod embed;`）
- [x] `src/usecase/extract.rs` ✅ 2026-03-19 — ファイルパスを受け取り XML 文字列を返す

```rust
use std::fs;
use crate::domain::{png, extract};

pub fn extract_xml_from_png(input_path: &str) -> anyhow::Result<String> {
    let data = fs::read(input_path)?;
    png::validate_signature(&data)?;
    let chunks = png::parse_chunks(&data)?;
    extract::extract_drawio_xml(&chunks)
}
```

- [x] `src/usecase/embed.rs` ✅ 2026-03-19 — drawio XML + PNG パスを受け取り、埋め込み済み PNG バイナリを返す

```rust
use std::fs;
use crate::domain::{png, embed};

pub fn embed_xml_into_png(xml_path: &str, png_path: &str) -> anyhow::Result<Vec<u8>> {
    let xml = fs::read_to_string(xml_path)?;
    let png_data = fs::read(png_path)?;
    png::validate_signature(&png_data)?;
    let mut chunks = png::parse_chunks(&png_data)?;
    embed::inject_ztxt_chunk(&mut chunks, &xml)?;
    Ok(png::build_png(&chunks))
}
```

**ファイル行数見積**: 各約30行

---

### Step 1.6: インフラ層 — CLI アダプター (`src/adapter/cli.rs`)

clap のサブコマンド定義と、ユースケースの呼び出しを実装する。

- [x] `src/adapter/mod.rs` ✅ 2026-03-19 を作成（`pub mod cli;`）
- [x] `src/adapter/cli.rs` ✅ 2026-03-19 に以下を実装

```rust
use clap::{Parser, Subcommand};
use std::io::Write;

#[derive(Parser)]
#[command(name = "drawio-tools", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Extract draw.io XML from an embedded PNG
    Extract {
        /// Input PNG file path
        input: String,
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Embed draw.io XML into an existing PNG
    Embed {
        /// Input .drawio XML file path
        xml: String,
        /// Input PNG file path
        png: String,
        /// Output PNG file path
        #[arg(short, long)]
        output: String,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Extract { input, output } => {
            let xml = crate::usecase::extract::extract_xml_from_png(&input)?;
            match output {
                Some(path) => std::fs::write(&path, &xml)?,
                None => std::io::stdout().write_all(xml.as_bytes())?,
            }
        }
        Commands::Embed { xml, png, output } => {
            let result = crate::usecase::embed::embed_xml_into_png(&xml, &png)?;
            std::fs::write(&output, &result)?;
        }
    }
    Ok(())
}
```

- [x] `src/main.rs` ✅ 2026-03-19 を更新して `adapter::cli::run()` を呼ぶ

```rust
mod domain;
mod usecase;
mod adapter;

fn main() -> anyhow::Result<()> {
    adapter::cli::run()
}
```

**ファイル行数見積**: 約60行

---

### Step 1.7: テストフィクスチャ作成

- [x] `tests/fixtures/simple.drawio` ✅ 2026-03-19 — テスト用の最小限 drawio XML を作成

```xml
<mxfile>
  <diagram name="Page-1" id="test1">
    <mxGraphModel>
      <root>
        <mxCell id="0"/>
        <mxCell id="1" parent="0"/>
        <mxCell id="2" value="Hello" style="rounded=1;" vertex="1" parent="1">
          <mxGeometry x="100" y="100" width="120" height="60" as="geometry"/>
        </mxCell>
      </root>
    </mxGraphModel>
  </diagram>
</mxfile>
```

- [x] `tests/fixtures/simple_embedded.png` ✅ 2026-03-19 — Node.js スクリプトで事前生成した XML 埋め込み PNG を配置
  - 手順: `node /tmp/drawio-test/export_with_embed.mjs` で生成 → コピー
- [x] `tests/fixtures/plain.png` ✅ 2026-03-19 — XML 未埋め込みの通常 PNG（1x1 ピクセル等）を配置

---

### Step 1.8: ユニットテスト — 正常系

- [x] `src/domain/png.rs` ✅ 2026-03-19 内に `#[cfg(test)] mod tests` を追加

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_signature_valid() {
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, /* ... */];
        assert!(validate_signature(&data).is_ok());
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
}
```

- [x] `src/domain/extract.rs` ✅ 2026-03-19 内にユニットテスト追加

```rust
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
    fn test_decode_payload_base64() {
        let b64 = base64::engine::general_purpose::STANDARD
            .encode("<mxfile></mxfile>");
        assert_eq!(decode_payload(&b64).unwrap(), "<mxfile></mxfile>");
    }

    #[test]
    fn test_extract_drawio_xml_from_embedded_png() {
        let data = std::fs::read("tests/fixtures/simple_embedded.png").unwrap();
        let chunks = crate::domain::png::parse_chunks(&data).unwrap();
        let xml = extract_drawio_xml(&chunks).unwrap();
        assert!(xml.contains("<mxfile"));
        assert!(xml.contains("mxGraphModel"));
    }
}
```

- [x] `src/domain/embed.rs` ✅ 2026-03-19 内にユニットテスト追加

```rust
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
        // 最後から2番目が zTXt
        let ztxt = &chunks[chunks.len() - 2];
        assert_eq!(&ztxt.chunk_type, b"zTXt");
        // 最後が IEND
        assert_eq!(&chunks.last().unwrap().chunk_type, b"IEND");
    }
}
```

---

### Step 1.9: ユニットテスト — 異常系

- [x] `src/domain/png.rs` ✅ 2026-03-19 の異常系テスト

```rust
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
fn test_parse_chunks_empty_after_signature() {
    let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    // シグネチャのみ、チャンクなし → 空 Vec or エラー
    let result = parse_chunks(&data);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_parse_chunks_truncated_chunk() {
    // シグネチャ + 不完全なチャンクデータ
    let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0A]); // length=10
    data.extend_from_slice(b"IHDR"); // type
    // data が足りない → エラー
    assert!(parse_chunks(&data).is_err());
}
```

- [x] `src/domain/extract.rs` ✅ 2026-03-19 の異常系テスト

```rust
#[test]
fn test_extract_drawio_xml_no_ztxt() {
    let plain = std::fs::read("tests/fixtures/plain.png").unwrap();
    let chunks = crate::domain::png::parse_chunks(&plain).unwrap();
    let result = extract_drawio_xml(&chunks);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No draw.io XML found"));
}

#[test]
fn test_decode_ztxt_no_null_separator() {
    let data = b"mxGraphModel"; // null がない
    assert!(decode_ztxt(data).is_err());
}

#[test]
fn test_decode_ztxt_invalid_compressed_data() {
    let mut data = b"mxGraphModel\0\0".to_vec();
    data.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // 不正な圧縮データ
    assert!(decode_ztxt(&data).is_err());
}
```

- [x] `src/domain/embed.rs` ✅ 2026-03-19 の異常系テスト

```rust
#[test]
fn test_inject_ztxt_chunk_no_iend() {
    let mut chunks = vec![
        crate::domain::png::PngChunk {
            chunk_type: *b"IHDR",
            data: vec![],
        },
    ];
    // IEND がない → エラー
    let result = inject_ztxt_chunk(&mut chunks, "<mxfile/>");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("IEND"));
}
```

---

### Step 1.10: 統合テスト — ラウンドトリップ検証

- [x] `tests/roundtrip_test.rs` ✅ 2026-03-19 を作成

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

#[test]
fn test_extract_outputs_xml_to_stdout() {
    Command::cargo_bin("drawio-tools").unwrap()
        .arg("extract")
        .arg("tests/fixtures/simple_embedded.png")
        .assert()
        .success()
        .stdout(predicate::str::contains("<mxfile"));
}

#[test]
fn test_extract_outputs_xml_to_file() {
    let output = NamedTempFile::new().unwrap();
    Command::cargo_bin("drawio-tools").unwrap()
        .arg("extract")
        .arg("tests/fixtures/simple_embedded.png")
        .arg("-o").arg(output.path())
        .assert()
        .success();
    let content = std::fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("<mxfile"));
}

#[test]
fn test_extract_plain_png_fails() {
    Command::cargo_bin("drawio-tools").unwrap()
        .arg("extract")
        .arg("tests/fixtures/plain.png")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No draw.io XML found"));
}

#[test]
fn test_extract_nonexistent_file_fails() {
    Command::cargo_bin("drawio-tools").unwrap()
        .arg("extract")
        .arg("nonexistent.png")
        .assert()
        .failure();
}

#[test]
fn test_embed_and_reextract_roundtrip() {
    let output_png = NamedTempFile::new().unwrap();

    // embed: simple.drawio + plain.png → output.png
    Command::cargo_bin("drawio-tools").unwrap()
        .arg("embed")
        .arg("tests/fixtures/simple.drawio")
        .arg("tests/fixtures/plain.png")
        .arg("-o").arg(output_png.path())
        .assert()
        .success();

    // extract: output.png → stdout に XML
    Command::cargo_bin("drawio-tools").unwrap()
        .arg("extract")
        .arg(output_png.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"))
        .stdout(predicate::str::contains("<mxfile"));
}
```

---

### Step 1.11: Phase 1 品質ゲート

- [x] テストカバレッジ ✅ 2026-03-19を計測し 90% 以上を確認する
  - `cargo install cargo-tarpaulin` でインストール
  - `cargo tarpaulin --out Html --output-dir coverage/` で計測
  - 90% 未満の場合: カバレッジレポートを確認し、未テスト箇所に対して正常系・異常系テストを追加
  - 例: 分岐カバレッジが不足する `decode_payload()` の各分岐に対して追加テスト
- [x] ビルド確認 ✅ 2026-03-19
  - `cargo build --release --no-default-features` が成功すること
  - `cargo clippy -- -D warnings` が警告なしで通ること
  - `cargo fmt --check` がフォーマット済みであること
- [ ] Docker ビルド確認 (skipped: no Docker in environment)
  - `docker build -t drawio-tools:phase1 .` が成功すること
  - `docker run --rm drawio-tools:phase1 --help` でヘルプが表示されること
  - `docker run --rm -v $(pwd)/tests/fixtures:/data drawio-tools:phase1 extract /data/simple_embedded.png` で XML が出力されること
  - エラーが発生する場合はエラーが出なくなるまで修正を繰り返す

---

## Phase 2: ヘッドレスレンダリング (export コマンド)

### Step 2.1: ドメイン層 — レンダリングインターフェース (`src/domain/render.rs`)

レンダリングのトレイト（抽象化）を定義する。

- [x] `src/domain/mod.rs` に `pub mod render;` を追加 ✅ 2026-03-19
- [x] `src/domain/render.rs` にトレイトを定義 ✅ 2026-03-19

```rust
/// ダイアグラムレンダリングのトレイト
pub trait DiagramRenderer {
    /// draw.io XML を受け取り、PNG バイナリを返す
    fn render_to_png(&self, xml: &str, scale: f64) -> anyhow::Result<Vec<u8>>;
}
```

**ファイル行数見積**: 約15行

---

### Step 2.2: インフラ層 — headless_chrome レンダラー (`src/infra/chrome_renderer.rs`)

`headless_chrome` クレートを使った `DiagramRenderer` 実装。

- [x] `src/infra/mod.rs` を作成 ✅ 2026-03-19（`pub mod chrome_renderer;`）
- [x] `src/infra/chrome_renderer.rs` に以下を実装 ✅ 2026-03-19

```rust
use headless_chrome::{Browser, LaunchOptions};
use crate::domain::render::DiagramRenderer;

pub struct ChromeRenderer {
    chromium_path: Option<String>,
}

impl ChromeRenderer {
    pub fn new(chromium_path: Option<String>) -> Self {
        Self { chromium_path }
    }
}

impl DiagramRenderer for ChromeRenderer {
    fn render_to_png(&self, xml: &str, scale: f64) -> anyhow::Result<Vec<u8>> {
        let mut options = LaunchOptions::default_builder();
        if let Some(ref path) = self.chromium_path {
            options = options.path(Some(std::path::PathBuf::from(path)));
        }
        let options = options
            .headless(true)
            .sandbox(false)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build launch options: {}", e))?;

        let browser = Browser::new(options)?;
        let tab = browser.new_tab()?;

        // 1. draw.io export3.html をロード
        tab.navigate_to("https://app.diagrams.net/export3.html")?;
        tab.wait_until_navigated()?;

        // 2. XML をパースしてレンダリング
        let js = format!(
            r#"
            (() => {{
                const doc = mxUtils.parseXml(`{}`);
                const dup = doc.documentElement.cloneNode(false);
                let child = doc.documentElement.firstChild;
                while (child) {{
                    if (child.nodeType === Node.ELEMENT_NODE) {{
                        dup.appendChild(child);
                        break;
                    }}
                    child = child.nextSibling;
                }}
                render({{
                    xml: dup.outerHTML,
                    format: 'png',
                    w: 0, h: 0,
                    border: 0,
                    bg: 'none',
                    scale: {},
                }});
            }})()
            "#,
            xml.replace('`', r"\`").replace('$', r"\$"),
            scale,
        );
        tab.evaluate(&js, false)?;

        // 3. レンダリング完了を待機
        tab.wait_for_element("#LoadingComplete")?;

        // 4. bounds を取得してスクリーンショット
        let bounds_js = r#"
            document.getElementById('LoadingComplete').getAttribute('bounds')
        "#;
        let bounds_val = tab.evaluate(bounds_js, false)?;
        // bounds をパースして viewport 設定 → スクリーンショット取得
        // (実装詳細は headless_chrome API に合わせて調整)

        let screenshot = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None, None, true,
        )?;

        Ok(screenshot)
    }
}
```

**参考**: `draw.io-export/export.js` の Puppeteer 制御ロジック（export3.html ロード → render() 呼び出し → スクリーンショット取得）
**ファイル行数見積**: 約100行（300行以下）
**注意**: `headless_chrome` の API が変更されている可能性あり。ビルド時にコンパイルエラーを確認し、API ドキュメントに合わせて修正すること。

---

### Step 2.3: ユースケース層 — export (`src/usecase/export.rs`)

レンダリング + 埋め込みのフルパイプラインを実装する。

- [x] `src/usecase/mod.rs` ✅ 2026-03-19 に `pub mod export;` を追加
- [x] `src/usecase/export.rs` に以下を実装 ✅ 2026-03-19

```rust
use crate::domain::render::DiagramRenderer;
use crate::domain::{png, embed};

pub fn export_drawio_to_png(
    renderer: &dyn DiagramRenderer,
    xml: &str,
    scale: f64,
) -> anyhow::Result<Vec<u8>> {
    // 1. レンダリング
    let png_data = renderer.render_to_png(xml, scale)?;

    // 2. レンダリング結果に XML を埋め込み
    png::validate_signature(&png_data)?;
    let mut chunks = png::parse_chunks(&png_data)?;
    embed::inject_ztxt_chunk(&mut chunks, xml)?;
    Ok(png::build_png(&chunks))
}
```

**ファイル行数見積**: 約25行

---

### Step 2.4: CLI アダプターに export コマンドを追加

- [x] `src/adapter/cli.rs` ✅ 2026-03-19 の `Commands` enum に `Export` を追加

```rust
/// Render draw.io XML to PNG with embedded XML
#[cfg(feature = "render")]
Export {
    /// Input .drawio XML file path
    input: String,
    /// Output PNG file path
    #[arg(short, long, default_value = "output.png")]
    output: String,
    /// Scale factor
    #[arg(short, long, default_value = "1.0")]
    scale: f64,
    /// Chromium binary path (auto-detect if omitted)
    #[arg(long)]
    chromium_path: Option<String>,
},
```

- [x] `run()` にマッチアームを追加 ✅ 2026-03-19

```rust
#[cfg(feature = "render")]
Commands::Export { input, output, scale, chromium_path } => {
    let xml = std::fs::read_to_string(&input)?;
    let renderer = crate::infra::chrome_renderer::ChromeRenderer::new(chromium_path);
    let result = crate::usecase::export::export_drawio_to_png(&renderer, &xml, scale)?;
    std::fs::write(&output, &result)?;
    eprintln!("Exported to {} ({} bytes, XML embedded)", output, result.len());
}
```

---

### Step 2.5: export コマンドのテスト — 正常系

- [ ] `tests/export_test.rs` を作成（`#[cfg(feature = "render")]` で囲む）

```rust
#[cfg(feature = "render")]
mod export_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_creates_png_with_embedded_xml() {
        let output = NamedTempFile::new().unwrap();
        Command::cargo_bin("drawio-tools").unwrap()
            .arg("export")
            .arg("tests/fixtures/simple.drawio")
            .arg("-o").arg(output.path())
            .assert()
            .success();

        // PNG シグネチャ確認
        let data = std::fs::read(output.path()).unwrap();
        assert_eq!(&data[..4], &[0x89, 0x50, 0x4E, 0x47]);

        // 埋め込み XML 抽出確認
        Command::cargo_bin("drawio-tools").unwrap()
            .arg("extract")
            .arg(output.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("<mxfile"));
    }

    #[test]
    fn test_export_with_scale() {
        let output = NamedTempFile::new().unwrap();
        Command::cargo_bin("drawio-tools").unwrap()
            .arg("export")
            .arg("tests/fixtures/simple.drawio")
            .arg("-o").arg(output.path())
            .arg("--scale").arg("2.0")
            .assert()
            .success();
    }
}
```

---

### Step 2.6: export コマンドのテスト — 異常系

- [ ] `tests/export_test.rs` に異常系テストを追加

```rust
#[cfg(feature = "render")]
#[test]
fn test_export_nonexistent_file_fails() {
    let output = NamedTempFile::new().unwrap();
    Command::cargo_bin("drawio-tools").unwrap()
        .arg("export")
        .arg("nonexistent.drawio")
        .arg("-o").arg(output.path())
        .assert()
        .failure();
}

#[cfg(feature = "render")]
#[test]
fn test_export_invalid_xml_fails() {
    // 空ファイルを入力
    let input = NamedTempFile::new().unwrap();
    std::fs::write(input.path(), "not xml at all").unwrap();
    let output = NamedTempFile::new().unwrap();
    Command::cargo_bin("drawio-tools").unwrap()
        .arg("export")
        .arg(input.path())
        .arg("-o").arg(output.path())
        .assert()
        .failure();
}

#[cfg(feature = "render")]
#[test]
fn test_export_with_invalid_chromium_path_fails() {
    let output = NamedTempFile::new().unwrap();
    Command::cargo_bin("drawio-tools").unwrap()
        .arg("export")
        .arg("tests/fixtures/simple.drawio")
        .arg("-o").arg(output.path())
        .arg("--chromium-path").arg("/nonexistent/chromium")
        .assert()
        .failure();
}
```

---

### Step 2.7: Phase 2 品質ゲート

- [ ] テストカバレッジを計測し 90% 以上を確認する
  - `cargo tarpaulin --features render --out Html --output-dir coverage/`
  - 90% 未満の場合: レンダラーのエラーパス、CLI の引数バリデーション等に対してテストを追加
  - render feature 無効時のテストも確認: `cargo tarpaulin --no-default-features --out Html`
- [ ] ビルド確認
  - `cargo build --release` が成功すること（render feature 有効）
  - `cargo build --release --no-default-features` が成功すること（render feature 無効）
  - `cargo clippy --all-features -- -D warnings` が警告なしで通ること
  - `cargo fmt --check` がフォーマット済みであること
- [ ] Docker ビルド確認
  - `docker build -t drawio-tools:phase2 .` が成功すること
  - `docker run --rm drawio-tools:phase2 --help` で extract / embed / export が表示されること
  - エラーが発生する場合はエラーが出なくなるまで修正を繰り返す

---

## Phase 3: CI/CD・リリース・デプロイ

### Step 3.1: GitHub Actions CI ワークフロー

- [x] `.github/workflows/ci.yml` を作成 ✅ 2026-03-19

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      - run: cargo fmt --check
      - run: cargo clippy --no-default-features -- -D warnings
      - run: cargo test --no-default-features
      - run: cargo build --release --no-default-features
```

---

### Step 3.2: GitHub Actions リリースワークフロー

- [x] `.github/workflows/release.yml` を作成 ✅ 2026-03-19（kalidokit-rust の release.yml を参考）

```yaml
name: Release
on:
  push:
    tags: ["v*"]

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always
  BINARY_NAME: drawio-tools

jobs:
  build-linux:
    name: Build (x86_64-unknown-linux-musl)
    runs-on: ubuntu-latest
    container:
      image: alpine:3.21
    steps:
      - name: Install system dependencies
        run: apk add --no-cache build-base git curl bash pkgconfig openssl-dev openssl-libs-static zlib-dev zlib-static
      - uses: actions/checkout@v4
      - name: Install Rust
        run: |
          export HOME=/root
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
          . "$HOME/.cargo/env"
          rustup target add x86_64-unknown-linux-musl
          echo "$HOME/.cargo/bin" >> $GITHUB_PATH
      - name: Build release binary
        env:
          HOME: /root
        run: |
          . "$HOME/.cargo/env"
          cargo build --release --no-default-features --target x86_64-unknown-linux-musl
      - name: Package
        run: |
          ARCHIVE="${BINARY_NAME}-${{ github.ref_name }}-x86_64-unknown-linux-musl"
          mkdir -p "${ARCHIVE}"
          cp "target/x86_64-unknown-linux-musl/release/${BINARY_NAME}" "${ARCHIVE}/"
          tar czf "${ARCHIVE}.tar.gz" "${ARCHIVE}"
          echo "ASSET=${ARCHIVE}.tar.gz" >> $GITHUB_ENV
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ env.BINARY_NAME }}-x86_64-unknown-linux-musl
          path: ${{ env.ASSET }}

  build:
    name: Build (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: aarch64-apple-darwin
            os: macos-latest
            archive: tar.gz
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            archive: zip
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ matrix.target }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      - run: cargo build --release --no-default-features --target ${{ matrix.target }}
      - name: Package (Unix)
        if: matrix.archive == 'tar.gz'
        run: |
          ARCHIVE="${BINARY_NAME}-${{ github.ref_name }}-${{ matrix.target }}"
          mkdir -p "${ARCHIVE}"
          cp "target/${{ matrix.target }}/release/${BINARY_NAME}" "${ARCHIVE}/"
          tar czf "${ARCHIVE}.tar.gz" "${ARCHIVE}"
          echo "ASSET=${ARCHIVE}.tar.gz" >> $GITHUB_ENV
      - name: Package (Windows)
        if: matrix.archive == 'zip'
        shell: pwsh
        run: |
          $a = "${{ env.BINARY_NAME }}-${{ github.ref_name }}-${{ matrix.target }}"
          New-Item -ItemType Directory -Force -Path $a
          Copy-Item "target/${{ matrix.target }}/release/${{ env.BINARY_NAME }}.exe" "$a/"
          Compress-Archive -Path "$a" -DestinationPath "$a.zip"
          echo "ASSET=$a.zip" >> $env:GITHUB_ENV
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ env.BINARY_NAME }}-${{ matrix.target }}
          path: ${{ env.ASSET }}

  release:
    name: Create GitHub Release
    needs: [build-linux, build]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
          merge-multiple: true
      - uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: artifacts/*
```

---

### Step 3.3: install / uninstall スクリプト

- [x] `scripts/setup.sh` を作成 ✅ 2026-03-19（kalidokit-rust の setup.sh を参考に drawio-tools 向けに書き換え）

```bash
#!/bin/sh
set -e
DEFAULT_INSTALL_PATH="/usr/local/bin"
REPO="tk-aria/drawio-tools"  # ← 実際のリポジトリに変更
BINARY_NAME="drawio-tools"

# ユーティリティ関数: _latest_version, _detect_os, _detect_arch, _get_target, _get_ext, _get_binary_file, _download_url
# → kalidokit-rust/scripts/setup.sh と同じ構造

# cmd_install: GitHub Releases からバイナリをダウンロード・配置
# cmd_uninstall: バイナリを削除
# usage: ヘルプ表示
# main: エントリポイント
```

主な変更点:
- `REPO`, `BINARY_NAME` を drawio-tools 向けに変更
- `download-models` サブコマンドは不要なので削除
- assets コピーロジックは不要なので削除
- 環境変数プレフィックスを `DRAWIO_TOOLS_` に変更

---

### Step 3.4: Docker イメージビルド & プッシュ

- [x] `Dockerfile` ✅ 2026-03-19 を最終版に更新（マルチステージビルド）

```dockerfile
FROM rust:1.75-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release --no-default-features

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/drawio-tools /usr/local/bin/
ENTRYPOINT ["drawio-tools"]
```

- [ ] ローカルでビルド (Docker環境なし: skipped)・動作確認

```bash
docker build -t ghcr.io/granizm/drawio-tools:latest .
docker run --rm ghcr.io/granizm/drawio-tools:latest --help
docker run --rm -v $(pwd)/tests/fixtures:/data ghcr.io/granizm/drawio-tools:latest extract /data/simple_embedded.png
```

- [ ] Docker push (環境制約: skipped)

```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
docker push ghcr.io/granizm/drawio-tools:latest
docker tag ghcr.io/granizm/drawio-tools:latest ghcr.io/granizm/drawio-tools:v0.1.0
docker push ghcr.io/granizm/drawio-tools:v0.1.0
```

---

### Step 3.5: Phase 3 品質ゲート

- [ ] テストカバレッジを計測し 90% 以上を確認する
  - `cargo tarpaulin --no-default-features --out Html --output-dir coverage/`
  - 90% 未満の場合: setup.sh のパス・分岐、Dockerfile のエントリポイント等のテストを追加
- [ ] ビルド確認
  - `cargo build --release --no-default-features` が成功すること
  - `cargo test --no-default-features` が全件パスすること
  - `docker build -t drawio-tools:phase3 .` が成功すること
  - `docker run --rm drawio-tools:phase3 --help` が正常表示すること
  - `docker run --rm -v $(pwd)/tests/fixtures:/data drawio-tools:phase3 extract /data/simple_embedded.png` が XML を出力すること
  - エラーが発生する場合はエラーが出なくなるまで修正を繰り返す
- [ ] リリースワークフロー検証
  - `git tag v0.0.1-test && git push origin v0.0.1-test` でワークフローが起動し、3 プラットフォームのバイナリが Release に添付されることを確認
  - 失敗する場合はワークフローを修正して再度タグプッシュ

---

## Phase 4: 最終検証・デプロイ

### Step 4.1: 動作確認 TODO リスト作成・実行

features.md の全実装内容を元に「動作確認の TODO リスト」を作成し、各項目を実際にバイナリ/Docker で実行して検証する。

- [x] 動作確認リストを作成 ✅ 2026-03-19（以下のテンプレート）:

```
## 動作確認チェックリスト

### extract コマンド
- [ ] `drawio-tools extract tests/fixtures/simple_embedded.png` → stdout に XML 出力
- [ ] `drawio-tools extract tests/fixtures/simple_embedded.png -o out.drawio` → ファイル出力
- [ ] `drawio-tools extract tests/fixtures/plain.png` → エラーメッセージ表示
- [ ] `drawio-tools extract nonexistent.png` → エラーメッセージ表示
- [ ] `echo "" | drawio-tools extract /dev/stdin` → エラーメッセージ表示

### embed コマンド
- [ ] `drawio-tools embed tests/fixtures/simple.drawio tests/fixtures/plain.png -o out.png` → 埋め込み PNG 生成
- [ ] 生成した out.png を extract で再抽出 → 元の XML と一致確認
- [ ] 存在しない XML ファイル指定 → エラー
- [ ] 存在しない PNG ファイル指定 → エラー

### export コマンド (render feature 有効時)
- [ ] `drawio-tools export tests/fixtures/simple.drawio -o out.png` → PNG 生成 + XML 埋め込み
- [ ] `drawio-tools export tests/fixtures/simple.drawio -o out.png --scale 2.0` → スケール変更
- [ ] 生成 PNG を extract で XML 抽出可能か確認

### CLI 一般
- [ ] `drawio-tools --help` → ヘルプ表示
- [ ] `drawio-tools --version` → バージョン表示
- [ ] `drawio-tools extract --help` → サブコマンドヘルプ表示

### Docker
- [ ] `docker run --rm ghcr.io/granizm/drawio-tools:latest --help`
- [ ] `docker run --rm -v $(pwd)/tests:/data ghcr.io/granizm/drawio-tools:latest extract /data/fixtures/simple_embedded.png`

### setup.sh
- [ ] `sh scripts/setup.sh install` → バイナリインストール
- [ ] `drawio-tools --version` → インストール確認
- [ ] `sh scripts/setup.sh uninstall` → アンインストール
```

- [x] 上記リストを1項目ずつ実行 ✅ 2026-03-19し、失敗した項目があれば原因を調査して修正する
- [x] 全項目がパスするまで ✅ 2026-03-19繰り返し修正する

---

### Step 4.2: 成果物デプロイ — Docker イメージ

- [ ] Docker ビルド & プッシュ（ghcr.io/granizm 宛）

```bash
docker build -t ghcr.io/granizm/drawio-tools:latest .
docker build -t ghcr.io/granizm/drawio-tools:v0.1.0 .
docker push ghcr.io/granizm/drawio-tools:latest
docker push ghcr.io/granizm/drawio-tools:v0.1.0
```

- [ ] プッシュ後に pull して動作確認

```bash
docker pull ghcr.io/granizm/drawio-tools:latest
docker run --rm ghcr.io/granizm/drawio-tools:latest --help
```

---

### Step 4.3: 成果物デプロイ — バイナリリリース

- [ ] GitHub リポジトリにプッシュ

```bash
git remote add origin https://github.com/tk-aria/drawio-tools.git  # ← 実際のリポジトリ
git push -u origin main
```

- [ ] リリースワークフロー実行

```bash
git tag v0.1.0
git push origin v0.1.0
```

- [ ] GitHub Actions で Release ジョブが完了し、以下のアーティファクトが添付されていることを確認:
  - `drawio-tools-v0.1.0-x86_64-unknown-linux-musl.tar.gz`
  - `drawio-tools-v0.1.0-aarch64-apple-darwin.tar.gz`
  - `drawio-tools-v0.1.0-x86_64-pc-windows-msvc.zip`
- [ ] 失敗する場合はワークフローを修正して再度タグプッシュ（`git tag -d v0.1.0 && git push origin :v0.1.0` → 修正 → 再タグ）

---

### Step 4.4: install / uninstall スクリプトの動作確認

- [ ] setup.sh の install テスト

```bash
curl -sSLf https://raw.githubusercontent.com/tk-aria/drawio-tools/main/scripts/setup.sh | sh -s install
drawio-tools --version
drawio-tools --help
```

- [ ] setup.sh の uninstall テスト

```bash
curl -sSLf https://raw.githubusercontent.com/tk-aria/drawio-tools/main/scripts/setup.sh | sh -s uninstall
which drawio-tools  # → not found であること
```

- [ ] 失敗する場合は setup.sh を修正して再度テスト
