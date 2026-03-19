# Statement of Work (SoW): drawio-tools

- **Project**: drawio-tools — Draw.io PNG/XML 双方向変換 CLI ツール
- **Version**: 1.0
- **Date**: 2026-03-19
- **Reference**: RFC-001

## 1. プロジェクト概要

draw.io ダイアグラムファイルと PNG 画像間の双方向変換を行う Rust 製 CLI ツールの開発。
PNG zTXt チャンクを介した XML の抽出・埋め込み、および headless Chromium を用いたレンダリングを提供する。

## 2. スコープ

### 2.1 スコープ内

| ID | 成果物 | 説明 |
|---|---|---|
| D-1 | `drawio-tools` CLI バイナリ | extract / embed / export サブコマンドを持つシングルバイナリ |
| D-2 | RFC ドキュメント | 技術仕様・設計方針（RFC-001） |
| D-3 | SoW ドキュメント | 本文書 |
| D-4 | 統合テスト | テスト用 .drawio ファイル + テストスイート |
| D-5 | CI 設定 | GitHub Actions ワークフロー（ビルド・テスト・リリース） |

### 2.2 スコープ外

- GUI / Web UI
- draw.io エディター機能の再実装
- AI/OCR ベースの画像→ダイアグラム変換
- Visio (.vsdx) 形式の対応
- SaaS / マネージドサービスとしてのホスティング

## 3. 作業分解構造（WBS）

### Phase 1: PNG チャンク操作（純 Rust）

| ID | タスク | 成果物 | 受入基準 |
|---|---|---|---|
| P1-1 | PNG バイナリパーサー実装 | `src/png.rs` | PNG シグネチャ検証、全チャンクの走査が可能 |
| P1-2 | zTXt/tEXt/iTXt 抽出ロジック | `src/extract.rs` | draw.io 公式エクスポート PNG から XML を正しく抽出 |
| P1-3 | データデコーダー | extract.rs 内 | 生 XML / URL エンコード / Base64 の 3 形式を自動判定・デコード |
| P1-4 | zTXt チャンク生成・注入 | `src/embed.rs` | 任意の PNG に draw.io XML を埋め込み、draw.io で再編集可能 |
| P1-5 | CRC32 計算 | png.rs 内 | `crc32fast` クレートを使用、注入チャンクの CRC が正しい |

### Phase 2: CLI フレームワーク・テスト

| ID | タスク | 成果物 | 受入基準 |
|---|---|---|---|
| P2-1 | CLI エントリポイント | `src/main.rs` | `clap` derive で extract / embed / export サブコマンドを定義 |
| P2-2 | extract コマンド | main.rs + extract.rs | `-o` でファイル出力、省略時は stdout、終了コード適切 |
| P2-3 | embed コマンド | main.rs + embed.rs | `.drawio` + `.png` → 埋め込み PNG を出力 |
| P2-4 | テスト用 drawio ファイル作成 | `tests/fixtures/` | 最低 3 種類のテストダイアグラム |
| P2-5 | 統合テスト | `tests/` | extract → embed → re-extract のラウンドトリップ検証 |
| P2-6 | エラーハンドリング | 全モジュール | 不正 PNG / XML 未埋め込み PNG / 破損ファイル時の明確なエラーメッセージ |

### Phase 3: ヘッドレスレンダリング

| ID | タスク | 成果物 | 受入基準 |
|---|---|---|---|
| P3-1 | headless_chrome 統合 | `src/render.rs` | Chromium を起動し draw.io export3.html をロード |
| P3-2 | mxGraph レンダリング | render.rs | drawio XML を PNG 画像として正しくレンダリング |
| P3-3 | export コマンド | main.rs + render.rs | レンダリング + XML 埋め込みのフルパイプライン |
| P3-4 | Chromium 未検出時フォールバック | render.rs | 明確なエラーメッセージ + 外部コマンドフォールバックオプション |
| P3-5 | feature flag 分離 | Cargo.toml | `--no-default-features` で Phase 1-2 のみのスリムビルド可能 |

## 4. 技術要件

### 4.1 ビルド環境

| 項目 | 要件 |
|---|---|
| Rust Edition | 2021 |
| MSRV | 1.75 |
| ターゲット | `x86_64-unknown-linux-musl`（静的リンク） |
| CI | GitHub Actions |

### 4.2 依存クレート一覧

| クレート | バージョン | 用途 | Phase |
|---|---|---|---|
| `clap` | 4.x (derive) | CLI 引数解析 | 1 |
| `flate2` | 1.x | zlib inflate/deflate | 1 |
| `crc32fast` | 1.x | PNG CRC32 計算 | 1 |
| `anyhow` | 1.x | エラーハンドリング | 1 |
| `percent-encoding` | 2.x | URL デコード | 1 |
| `base64` | 0.22.x | Base64 デコード | 1 |
| `headless_chrome` | 1.x | Chromium 制御 | 3 |

### 4.3 テスト要件

| テスト種別 | 対象 | 基準 |
|---|---|---|
| ユニットテスト | PNG パーサー、CRC32、エンコーダー/デコーダー | カバレッジ 80% 以上 |
| 統合テスト | extract / embed ラウンドトリップ | 入出力一致 |
| E2E テスト | export コマンド（Phase 3） | PNG 出力 + XML 抽出可能 |

## 5. ディレクトリ構成

```
drawio-tools/
├── Cargo.toml
├── docs/
│   ├── RFC.md
│   └── SOW.md
├── src/
│   ├── main.rs
│   ├── png.rs
│   ├── extract.rs
│   ├── embed.rs
│   └── render.rs
├── tests/
│   ├── fixtures/
│   │   ├── simple.drawio
│   │   ├── simple_embedded.png
│   │   └── multipage.drawio
│   ├── extract_test.rs
│   ├── embed_test.rs
│   └── roundtrip_test.rs
└── .github/
    └── workflows/
        └── ci.yml
```

## 6. マイルストーン

| マイルストーン | Phase | 完了基準 |
|---|---|---|
| M1: extract + embed 動作 | Phase 1-2 | extract / embed が統合テストを通過 |
| M2: CI パイプライン稼働 | Phase 2 | GitHub Actions でビルド・テスト・lint が通過 |
| M3: export コマンド動作 | Phase 3 | headless Chromium でレンダリング + 埋め込みが動作 |
| M4: v0.1.0 リリース | Phase 3 | musl 静的リンクバイナリの GitHub Releases 公開 |

## 7. 受入基準

### 7.1 機能要件

- [ ] `drawio-tools extract` で XML 埋め込み PNG から XML を正しく抽出できる
- [ ] `drawio-tools embed` で PNG に XML を埋め込み、draw.io で再編集可能な PNG を出力できる
- [ ] `drawio-tools export` で drawio XML から PNG をレンダリングし XML を埋め込める
- [ ] `-o` オプション省略時は標準出力に出力される
- [ ] XML 未埋め込み PNG に対して明確なエラーメッセージを表示する

### 7.2 非機能要件

- [ ] extract / embed の実行時間が 100ms 以内（一般的なダイアグラム）
- [ ] シングルバイナリで配布可能（musl 静的リンク）
- [ ] `--help` で各コマンドの使い方が表示される

## 8. 前提条件・制約

- Phase 3（export）は実行環境に Chromium がインストールされていることが前提
- draw.io の PNG 埋め込み仕様（zTXt チャンク、keyword: `mxGraphModel`）が今後も維持されることを前提とする
- AI/OCR ベースの画像認識変換はスコープ外
