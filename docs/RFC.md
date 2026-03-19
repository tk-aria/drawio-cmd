# RFC: drawio-tools — Draw.io PNG/XML 双方向変換 CLI ツール

- **RFC ID**: RFC-001
- **Status**: Draft
- **Author**: drawio-tools contributors
- **Created**: 2026-03-19

## 1. 概要

draw.io (diagrams.net) のダイアグラムファイル（`.drawio` / `.xml`）と PNG 画像間の双方向変換を行う Rust 製 CLI ツール。

draw.io が PNG ファイルの zTXt チャンクに XML データを埋め込む仕様を活用し、PNG からの XML 抽出、および XML の PNG への埋め込み・レンダリングを実現する。

## 2. 背景と動機

### 2.1 課題

- draw.io Desktop CLI は GUI 環境（Xvfb 等）が必要で、サーバー/CI 環境での運用が煩雑
- Node.js ベースの `draw.io-export` npm パッケージは XML 埋め込み機能を持たない
- PNG から XML を抽出するための標準的な CLI ツールが存在しない
- 既存ツールは Node.js / npm 依存で、デプロイ・配布が重い

### 2.2 目標

- **シングルバイナリ配布**: Node.js / npm 不要
- **高速起動**: extract / embed は Chromium 不要で即座に実行
- **Unix 哲学**: stdin/stdout 対応、パイプライン連携可能
- **静的リンク**: musl ターゲットで Docker イメージ最小化

## 3. 技術仕様

### 3.1 PNG zTXt チャンクの構造

draw.io は PNG エクスポート時に「Include a copy of my diagram」オプションが有効な場合、
以下の構造で XML を PNG に埋め込む:

```
PNG File
├── Signature (8 bytes): 89 50 4E 47 0D 0A 1A 0A
├── IHDR chunk
├── IDAT chunk(s)
├── zTXt chunk                    ← draw.io XML が格納される
│   ├── Length (4 bytes, BE)
│   ├── Type: "zTXt" (4 bytes)
│   ├── Data:
│   │   ├── Keyword: "mxGraphModel\0"
│   │   ├── Compression Method: 0x00 (deflate)
│   │   └── Compressed Data: deflate(XML)
│   └── CRC32 (4 bytes)
└── IEND chunk
```

### 3.2 データエンコーディング

解凍後のデータは以下のいずれかの形式:

| 形式 | 判定方法 | デコード |
|---|---|---|
| 生 XML | `<` で始まる | そのまま使用 |
| URL エンコード | `%3C` を含む | `percent_decode` |
| Base64 エンコード | `PD` or `eN` で始まる | base64 decode |

### 3.3 チャンクタイプ対応

| チャンクタイプ | 圧縮 | 対応状況 |
|---|---|---|
| `zTXt` | deflate 圧縮 | 主要（draw.io 標準） |
| `tEXt` | なし | フォールバック |
| `iTXt` | オプション | 国際化テキスト対応 |

## 4. CLI インターフェース設計

### 4.1 サブコマンド

```
drawio-tools <COMMAND>

Commands:
  extract   PNG から draw.io XML を抽出
  embed     既存 PNG に draw.io XML を埋め込む
  export    draw.io XML を PNG にレンダリング（XML 埋め込み付き）
```

### 4.2 extract

```bash
# 標準出力に出力
drawio-tools extract diagram.png

# ファイルに出力
drawio-tools extract diagram.png -o diagram.drawio

# パイプ連携
drawio-tools extract diagram.png | xmllint --format -
```

### 4.3 embed

```bash
# 既存 PNG に XML を注入
drawio-tools embed diagram.drawio image.png -o output.png
```

### 4.4 export

```bash
# XML → PNG レンダリング + XML 埋め込み
drawio-tools export diagram.drawio -o output.png

# スケーリング指定
drawio-tools export diagram.drawio -o output.png --scale 2.0
```

## 5. アーキテクチャ

```
┌─────────────────────────────────────────┐
│              drawio-tools               │
├─────────┬──────────┬────────────────────┤
│ extract │  embed   │      export        │
│         │          │  ┌──────────────┐  │
│  PNG    │  PNG     │  │ headless     │  │
│  Parser │  Builder │  │ _chrome      │  │
│    +    │    +     │  │   +          │  │
│  zlib   │  zlib    │  │ export3.html │  │
│  inflate│  deflate │  │   +          │  │
│         │    +     │  │ PNG Builder  │  │
│         │  CRC32   │  └──────────────┘  │
├─────────┴──────────┴────────────────────┤
│              png.rs (共通)               │
│  PNGチャンク解析 / 構築 / CRC32計算       │
└─────────────────────────────────────────┘
```

### 5.1 モジュール構成

```
src/
├── main.rs          CLI エントリポイント（clap）
├── png.rs           PNG チャンクパーサー / ビルダー
├── extract.rs       zTXt/tEXt/iTXt → XML 抽出
├── embed.rs         XML → zTXt チャンク生成・PNG 注入
└── render.rs        headless_chrome レンダリング
```

### 5.2 依存クレート

| クレート | 用途 | Phase |
|---|---|---|
| `clap` (4.x, derive) | CLI 引数解析 | 1 |
| `flate2` (1.x) | zlib inflate/deflate | 1 |
| `crc32fast` (1.x) | PNG CRC32 計算 | 1 |
| `anyhow` (1.x) | エラーハンドリング | 1 |
| `percent-encoding` (2.x) | URL デコード | 1 |
| `base64` (0.22.x) | Base64 デコード | 1 |
| `headless_chrome` (1.x) | Chromium 制御 | 3 |

## 6. 実装フェーズ

### Phase 1: PNG チャンク操作（純 Rust）

- PNG バイナリパーサー（シグネチャ検証、チャンク走査）
- zTXt / tEXt / iTXt チャンクからの XML 抽出
- URL-encoding / Base64 デコード対応
- zTXt チャンク生成 + PNG 注入（CRC32 計算含む）
- `extract` / `embed` コマンド実装

### Phase 2: CLI フレームワーク・テスト

- `clap` によるサブコマンド定義
- 統合テスト（テスト用 drawio ファイル同梱）
- エラーメッセージの整備

### Phase 3: ヘッドレスレンダリング

- `headless_chrome` クレートで Chromium 制御
- draw.io `export3.html` をロードして mxGraph レンダリング
- `export` コマンド実装（レンダリング → 埋め込みのフルパイプライン）
- Chromium 未インストール時のエラーハンドリング

## 7. リスクと緩和策

| リスク | 影響 | 緩和策 |
|---|---|---|
| `headless_chrome` のメンテナンス停滞 | Phase 3 が実装不可 | `chromiumoxide` へ切り替え、または外部コマンド (`npx drawio`) フォールバック |
| draw.io が zTXt 仕様を変更 | extract が動作しなくなる | draw.io OSS リポジトリのウォッチ、バージョニング対応 |
| Chromium バイナリサイズ | Docker イメージ肥大化 | Phase 1-2 のみの slim ビルドを feature flag で提供 |
| 大規模ダイアグラムでの OOM | レンダリング失敗 | Chromium のメモリ制限設定、タイムアウト |

## 8. 今後の拡張可能性

- SVG 出力対応
- PDF 出力対応
- 複数ページダイアグラムの個別エクスポート
- GitHub Actions / CI での利用を想定した quiet モード
- WebAssembly ビルド（ブラウザ上での extract/embed）

## 9. 参考資料

- [draw.io XML in PNG 公式ブログ](https://www.drawio.com/blog/xml-in-png)
- [PNG 仕様 - zTXt チャンク](https://www.w3.org/TR/png/#11zTXt)
- [pzl/drawio-read (Go 実装参考)](https://github.com/pzl/drawio-read)
- [draw.io-export npm パッケージ](https://www.npmjs.com/package/draw.io-export)
- [jgraph/draw-image-export2 公式エクスポートサーバー](https://github.com/jgraph/draw-image-export2)
