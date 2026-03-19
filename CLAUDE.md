# drawio-cmd

draw.io PNG/XML 双方向変換 CLI ツール。

## プロジェクト構成

```
src/
  domain/     PNG チャンク操作・XML 抽出/埋め込みロジック
  usecase/    ユースケース層
  adapter/    CLI (clap)
  infra/      Chrome レンダラー (render feature)
tests/        結合テスト + fixtures
example/      請求フロー図サンプル
```

## ビルド

```bash
# この環境では親ワークスペースの musl 設定を上書きするため --target 指定が必要
cargo build --no-default-features --target x86_64-unknown-linux-gnu
```

バイナリ: `target/x86_64-unknown-linux-gnu/debug/drawio-cmd`

## テスト

```bash
cargo test --no-default-features --target x86_64-unknown-linux-gnu
```

## カスタムコマンド

- `/update-diagram` — PNG 内の draw.io XML を抽出→編集→再埋め込みするワークフロー
