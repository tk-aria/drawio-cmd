# drawio-cmd

A Rust CLI tool for bidirectional conversion between Draw.io (diagrams.net) PNG images and XML diagrams.

Leverages draw.io's zTXt PNG chunk embedding to extract XML from PNG files and embed XML back into PNGs — all in a single static binary.

## 🔧 Features

| Command | Description | Chromium |
|---------|-------------|----------|
| `extract` | Extract draw.io XML from a PNG | Not required |
| `embed` | Embed draw.io XML into a PNG | Not required |
| `export` | Render draw.io XML to PNG with embedded XML | Required (`render` feature) |

## 📦 Installation

### curl (Linux / macOS)

```bash
curl -sSLf https://raw.githubusercontent.com/tk-aria/drawio-cmd/main/scripts/setup.sh | sh -s install
```

Custom install path:

```bash
DRAWIO_CMD_INSTALL_PATH=$HOME/.local/bin \
  curl -sSLf https://raw.githubusercontent.com/tk-aria/drawio-cmd/main/scripts/setup.sh | sh -s install
```

### Manual Download

Download the appropriate binary for your platform from [GitHub Releases](https://github.com/tk-aria/drawio-cmd/releases) and place it in your PATH.

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

## 🚀 Usage

### Extract XML from PNG

```bash
# Output to stdout
drawio-cmd extract diagram.png

# Output to file
drawio-cmd extract diagram.png -o diagram.drawio
```

### Embed XML into PNG

```bash
drawio-cmd embed diagram.drawio base.png -o output.png
```

### Roundtrip Verification

```bash
# Verify embed → extract produces the original XML
drawio-cmd embed diagram.drawio plain.png -o embedded.png
drawio-cmd extract embedded.png
```

## 🏗️ Architecture

```
src/
├── main.rs              # Entry point
├── domain/
│   ├── png.rs           # PNG chunk parsing & building
│   ├── extract.rs       # XML extraction from zTXt/tEXt/iTXt
│   ├── embed.rs         # zTXt chunk creation & injection
│   └── render.rs        # Rendering trait (render feature)
├── usecase/
│   ├── extract.rs       # PNG → XML use case
│   ├── embed.rs         # XML → PNG use case
│   └── export.rs        # Render + embed pipeline (render feature)
├── adapter/
│   └── cli.rs           # clap CLI adapter
└── infra/
    └── chrome_renderer.rs  # headless_chrome impl (render feature)
```

## 📋 Supported Formats

- **PNG chunk types**: zTXt, tEXt, iTXt
- **Keywords**: `mxGraphModel`, `mxfile`
- **Encoding auto-detection**: Raw XML, URL-encoded (`%3C`), Base64 (`PD...`, `eN...`)

## 🔨 Development

```bash
# Build
cargo build

# Test
cargo test

# Lint
cargo clippy -- -D warnings
cargo fmt --check
```

## ❌ Uninstall

```bash
curl -sSLf https://raw.githubusercontent.com/tk-aria/drawio-cmd/main/scripts/setup.sh | sh -s uninstall
```

## 📄 License

MIT
