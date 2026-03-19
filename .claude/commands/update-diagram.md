# update-diagram: drawio-cmd を使った PNG 内 draw.io XML の更新

drawio-cmd CLI を使い、PNG 画像に埋め込まれた draw.io XML を抽出・編集・再埋め込みするスキル。

## トリガー条件

以下のいずれかに該当するときにこのスキルを使用する:

- ユーザーが draw.io ダイアグラム(PNG)の内容を変更したいとき
- ユーザーが drawio XML を更新して PNG に反映したいとき
- ユーザーが「フロー図を更新して」「ダイアグラムにステップを追加して」等の指示をしたとき
- `*.drawio` ファイルまたは draw.io XML が埋め込まれた PNG を編集する必要があるとき

## 前提条件

- `drawio-cmd` バイナリがビルド済みであること
- 対象 PNG に draw.io XML が zTXt チャンクとして埋め込まれていること

## ワークフロー

```
┌─────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌────────────┐
│  1. Extract  │───▶│  2. Parse &  │───▶│  3. Re-render│───▶│  4. Embed    │───▶│  5. Commit │
│  XML from PNG│    │  Edit XML    │    │  PNG image   │    │  XML into PNG│    │  & Push    │
└─────────────┘    └──────────────┘    └──────────────┘    └──────────────┘    └────────────┘
```

### Step 1: PNG から XML を抽出

```bash
drawio-cmd extract <input.png> > /tmp/diagram.xml
```

### Step 2: XML を解析・編集

抽出した XML を読み、ユーザーの指示に従って編集する。

**draw.io XML の構造:**
```xml
<mxfile host="drawio-cmd">
  <diagram name="..." id="...">
    <mxGraphModel dx="1200" dy="800" ...>
      <root>
        <mxCell id="0"/>           <!-- ルート -->
        <mxCell id="1" parent="0"/> <!-- デフォルト親 -->

        <!-- スイムレーン -->
        <mxCell id="lane-xxx" value="レーン名"
          style="shape=swimlane;startSize=30;fillColor=#dae8fc;strokeColor=#6c8ebf;"
          vertex="1" parent="1">
          <mxGeometry x="40" y="80" width="1020" height="140" as="geometry"/>
        </mxCell>

        <!-- ノード（プロセス） -->
        <mxCell id="node-id" value="表示テキスト"
          style="rounded=1;whiteSpace=wrap;fillColor=#fff2cc;strokeColor=#d6b656;"
          vertex="1" parent="lane-xxx">
          <mxGeometry x="120" y="40" width="130" height="60" as="geometry"/>
        </mxCell>

        <!-- エッジ（矢印） -->
        <mxCell id="edge-id"
          style="edgeStyle=orthogonalEdgeStyle;"
          edge="1" source="source-id" target="target-id" parent="lane-xxx"/>

        <!-- レーン間エッジは parent="1" -->
        <mxCell id="cross-edge"
          style="edgeStyle=orthogonalEdgeStyle;"
          edge="1" source="node-a" target="node-b" parent="1"/>
      </root>
    </mxGraphModel>
  </diagram>
</mxfile>
```

**よく使うスタイル:**

| 要素 | style |
|------|-------|
| プロセス（黄） | `rounded=1;whiteSpace=wrap;fillColor=#fff2cc;strokeColor=#d6b656;` |
| 判断（赤ひし形） | `rhombus;whiteSpace=wrap;fillColor=#f8cecc;strokeColor=#b85450;` |
| 開始（緑丸） | `ellipse;fillColor=#d5e8d4;strokeColor=#82b366;aspect=fixed;` |
| 終了（赤丸） | `ellipse;fillColor=#f8cecc;strokeColor=#b85450;aspect=fixed;` |
| 青ハイライト | `rounded=1;whiteSpace=wrap;fillColor=#dae8fc;strokeColor=#6c8ebf;` |
| 赤ハイライト | `rounded=1;whiteSpace=wrap;fillColor=#f8cecc;strokeColor=#b85450;` |
| レーン（青） | `shape=swimlane;startSize=30;fillColor=#dae8fc;strokeColor=#6c8ebf;fontStyle=1;` |
| レーン（紫） | `shape=swimlane;startSize=30;fillColor=#e1d5e7;strokeColor=#9673a6;fontStyle=1;` |
| レーン（緑） | `shape=swimlane;startSize=30;fillColor=#d5e8d4;strokeColor=#82b366;fontStyle=1;` |
| 矢印 | `edgeStyle=orthogonalEdgeStyle;` |
| 破線矢印 | `edgeStyle=orthogonalEdgeStyle;dashed=1;` |

**編集パターン:**

- **ノード追加**: 新しい `<mxCell>` を適切な parent レーン内に追加し、既存エッジの source/target を更新
- **ノード削除**: `<mxCell>` とそれに接続するエッジを削除
- **テキスト変更**: `value` 属性を変更（改行は `&#xa;`）
- **レーン追加**: 新しいスイムレーンを追加し、既存レーンの y 座標を調整
- **レーン間エッジ**: `parent="1"` を使用（レーン内エッジはそのレーンを parent に）

### Step 3: PNG を再レンダリング

Node.js canvas で更新後のダイアグラムをレンダリングする。

```bash
node render-drawio.mjs /tmp/rendered.png
```

### Step 4: 更新 XML を PNG に埋め込み

```bash
drawio-cmd embed /tmp/updated.xml /tmp/rendered.png -o output.png
```

**重要:** `embed` コマンドは既存の mxGraphModel zTXt チャンクを自動削除してから新しい XML を挿入する（更新対応済み）。

### Step 5: 検証・コミット

```bash
# ラウンドトリップ検証
drawio-cmd extract output.png  # 更新された XML が出力されることを確認

# コミット
git add output.png source.drawio
git commit -m "update: diagram description"
git push
```

## 注意事項

- 座標系: レーン内ノードの座標はレーン左上が原点
- レーン間エッジ: `parent="1"` を使い、source/target でレーンをまたぐ接続を定義
- ID の一意性: 全 mxCell の `id` はファイル内で一意であること
- PNG と XML の同期: XML を更新したら必ず PNG も再レンダリングすること
