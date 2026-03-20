# Skill: drawio-cmd による draw.io PNG ダイアグラム更新

## 概要

draw.io 形式の XML が埋め込まれた PNG 画像を、`drawio-cmd` CLI を使ってプログラム的に更新するスキル。
ユーザーの自然言語指示（「ステップを追加して」「フローを変更して」等）を受けて、PNG 内の XML を抽出・編集・再埋め込みする。

## いつ使うか

以下のいずれかに該当する場合、このスキルを `/update-diagram` コマンドで実行する:

- draw.io のダイアグラム PNG を編集・更新したいとき
- `.drawio` ファイルの内容を変更して PNG に反映したいとき
- 「フロー図を修正して」「ダイアグラムにノードを追加して」等の指示があったとき
- プロジェクト内の `*.drawio` や draw.io XML 埋め込み PNG を検出したとき

## 必要ツール

| ツール | 用途 | 場所 |
|--------|------|------|
| `drawio-cmd` | XML 抽出/埋め込み | https://github.com/tk-aria/drawio-cmd |

### drawio-cmd インストール

```bash
# Linux (x86_64)
curl -fsSL https://github.com/tk-aria/drawio-cmd/releases/latest/download/drawio-cmd-x86_64-unknown-linux-musl.tar.gz \
  | tar xz -C /usr/local/bin/

# macOS (Apple Silicon)
curl -fsSL https://github.com/tk-aria/drawio-cmd/releases/latest/download/drawio-cmd-aarch64-apple-darwin.tar.gz \
  | tar xz -C /usr/local/bin/
```

## 実行手順

```
PNG → extract → XML → 編集 → embed → PNG(更新済み) → commit & push
```

### 1. XML 抽出

```bash
drawio-cmd extract <target.png> > /tmp/diagram.xml
```

### 2. XML 編集

抽出した XML を読み込み、ユーザーの指示に従って編集する。

### 3. 更新 XML を PNG に埋め込み

```bash
drawio-cmd embed /tmp/updated.xml <target.png> -o <output.png>
```

`embed` は既存の XML を自動で差し替える（古い zTXt チャンクを削除してから挿入）。

### 4. ラウンドトリップ検証

```bash
drawio-cmd extract <output.png>
# → 更新後の XML が出力されれば OK
```

### 5. コミット

```bash
git add <output.png> <source.drawio>
git commit -m "update: 変更内容の説明"
git push
```

## draw.io XML リファレンス

### 基本構造

```xml
<mxfile host="drawio-cmd">
  <diagram name="図名" id="diagram-1">
    <mxGraphModel dx="1200" dy="800" grid="1" ...>
      <root>
        <mxCell id="0"/>
        <mxCell id="1" parent="0"/>

        <!-- ノード: vertex="1" -->
        <mxCell id="node-1" value="表示テキスト"
          style="rounded=1;whiteSpace=wrap;fillColor=#fff2cc;strokeColor=#d6b656;"
          vertex="1" parent="1">
          <mxGeometry x="100" y="100" width="120" height="60" as="geometry"/>
        </mxCell>

        <!-- エッジ: edge="1" -->
        <mxCell id="edge-1"
          style="edgeStyle=orthogonalEdgeStyle;"
          edge="1" source="node-1" target="node-2" parent="1"/>
      </root>
    </mxGraphModel>
  </diagram>
</mxfile>
```

### スタイル早見表

| 要素 | style |
|------|-------|
| プロセス（黄） | `rounded=1;whiteSpace=wrap;fillColor=#fff2cc;strokeColor=#d6b656;` |
| 判断（ひし形） | `rhombus;whiteSpace=wrap;fillColor=#f8cecc;strokeColor=#b85450;` |
| 開始（緑丸） | `ellipse;fillColor=#d5e8d4;strokeColor=#82b366;aspect=fixed;` |
| 終了（赤丸） | `ellipse;fillColor=#f8cecc;strokeColor=#b85450;aspect=fixed;` |
| 青ボックス | `rounded=1;whiteSpace=wrap;fillColor=#dae8fc;strokeColor=#6c8ebf;` |
| レーン（青） | `shape=swimlane;startSize=30;fillColor=#dae8fc;strokeColor=#6c8ebf;fontStyle=1;` |
| レーン（紫） | `shape=swimlane;startSize=30;fillColor=#e1d5e7;strokeColor=#9673a6;fontStyle=1;` |
| レーン（緑） | `shape=swimlane;startSize=30;fillColor=#d5e8d4;strokeColor=#82b366;fontStyle=1;` |
| 矢印 | `edgeStyle=orthogonalEdgeStyle;` |
| 破線矢印 | `edgeStyle=orthogonalEdgeStyle;dashed=1;` |
| ラベル付きエッジ | `edgeStyle=orthogonalEdgeStyle;fontStyle=1;fontSize=11;` + `value="Yes"` |

### 編集ルール

- **改行**: `value` 属性内で `&#xa;` を使用
- **ID**: 全 `<mxCell>` の `id` はファイル内で一意
- **レーン内ノード**: `parent="lane-id"` — 座標はレーン左上が原点
- **レーン間エッジ**: `parent="1"` — source/target が異なるレーンのノードを指す
- **ノード追加時**: 前後のエッジの `source`/`target` も忘れずに更新
- **ノード削除時**: そのノードを参照するエッジも合わせて削除

## 実行例

```
ユーザー: 「請求フローに"Send Receipt"ステップを追加して」

→ drawio-cmd extract example/billing-flow.png > /tmp/diagram.xml
→ XML を編集: <mxCell id="send-receipt" ...> を追加、エッジを更新
→ drawio-cmd embed /tmp/updated.xml example/billing-flow.png -o example/billing-flow.png
→ drawio-cmd extract example/billing-flow.png  # 検証
→ git add & commit & push
```
