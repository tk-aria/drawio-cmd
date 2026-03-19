# drawio-cmd

Draw.io (diagrams.net) の PNG / XML 双方向変換を行う Rust 製 CLI ツール。

draw.io が PNG の zTXt チャンクに XML データを埋め込む仕様を活用し、PNG からの XML 抽出、および XML の PNG への埋め込みをシングルバイナリで実現します。

## 🔧 機能

| コマンド | 説明 | Chromium |
|----------|------|----------|
| `extract` | PNG から draw.io XML を抽出 | 不要 |
| `embed` | draw.io XML を PNG に埋め込み | 不要 |
| `export` | draw.io XML を PNG にレンダリング + XML 埋め込み | 必要 (`render` feature) |

## 📦 インストール

### curl (Linux / macOS)

```bash
curl -sSLf https://raw.githubusercontent.com/tk-aria/drawio-cmd/main/scripts/setup.sh | sh -s install
```

カスタムインストール先:

```bash
DRAWIO_CMD_INSTALL_PATH=$HOME/.local/bin \
  curl -sSLf https://raw.githubusercontent.com/tk-aria/drawio-cmd/main/scripts/setup.sh | sh -s install
```

### 手動ダウンロード

[GitHub Releases](https://github.com/tk-aria/drawio-cmd/releases) からプラットフォームに合ったバイナリをダウンロードし、`/usr/local/bin` などに配置してください。

```bash
# Linux (x86_64)
curl -sSLf https://github.com/tk-aria/drawio-cmd/releases/latest/download/drawio-cmd-latest-x86_64-unknown-linux-musl.tar.gz \
  | tar xz -C /tmp && sudo mv /tmp/drawio-cmd-*/drawio-cmd /usr/local/bin/

# macOS (Apple Silicon)
curl -sSLf https://github.com/tk-aria/drawio-cmd/releases/latest/download/drawio-cmd-latest-aarch64-apple-darwin.tar.gz \
  | tar xz -C /tmp && sudo mv /tmp/drawio-cmd-*/drawio-cmd /usr/local/bin/
```

### Cargo

```bash
cargo install --git https://github.com/tk-aria/drawio-cmd.git --no-default-features
```

### Docker

```bash
docker run --rm -v $(pwd):/data ghcr.io/tk-aria/drawio-cmd:latest extract /data/diagram.png
```

## 🚀 使い方

### PNG から XML を抽出

```bash
# stdout に出力
drawio-cmd extract diagram.png

# ファイルに出力
drawio-cmd extract diagram.png -o diagram.drawio
```

### XML を PNG に埋め込み

```bash
drawio-cmd embed diagram.drawio base.png -o output.png
```

### ラウンドトリップ確認

```bash
# 埋め込み → 抽出 で元の XML が取り出せることを確認
drawio-cmd embed diagram.drawio plain.png -o embedded.png
drawio-cmd extract embedded.png
```

## 🏗️ アーキテクチャ

```
src/
├── main.rs              # エントリポイント
├── domain/
│   ├── png.rs           # PNG チャンク解析・構築
│   ├── extract.rs       # zTXt/tEXt/iTXt から XML 抽出
│   ├── embed.rs         # zTXt チャンク生成・注入
│   └── render.rs        # レンダリングトレイト (render feature)
├── usecase/
│   ├── extract.rs       # PNG → XML ユースケース
│   ├── embed.rs         # XML → PNG ユースケース
│   └── export.rs        # レンダリング + 埋め込み (render feature)
├── adapter/
│   └── cli.rs           # clap CLI アダプター
└── infra/
    └── chrome_renderer.rs  # headless_chrome 実装 (render feature)
```

## 📋 対応フォーマット

- **PNG チャンクタイプ**: zTXt, tEXt, iTXt
- **キーワード**: `mxGraphModel`, `mxfile`
- **エンコーディング自動判定**: 生 XML, URL エンコード (`%3C`), Base64 (`PD...`, `eN...`)

## 🔨 開発

```bash
# ビルド
cargo build

# テスト
cargo test

# lint
cargo clippy -- -D warnings
cargo fmt --check
```

## ❌ アンインストール

```bash
curl -sSLf https://raw.githubusercontent.com/tk-aria/drawio-cmd/main/scripts/setup.sh | sh -s uninstall
```

## 📄 ライセンス

MIT
