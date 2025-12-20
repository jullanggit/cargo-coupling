# Web Visualization Command

Web UIを起動してカップリング分析結果を可視化します。

## 使用方法

```
/web [path] [options]
```

## 引数

- `path`: 分析対象のパス (デフォルト: ./src)
- `--port <PORT>`: ポート番号 (デフォルト: 3000)
- `--no-open`: ブラウザを自動で開かない

## 実行手順

1. プロジェクトをビルド
2. Web サーバーを起動
3. ブラウザで可視化を確認

```bash
cargo build --release
cargo run --release -- coupling --web $ARGUMENTS
```

## Web UI 機能

### グラフ操作
- **ノードクリック**: 隣接ノードをハイライト、中心に配置
- **エッジクリック**: 依存関係の方向を強調表示
- **背景クリック**: 選択解除

### キーボードショートカット
| キー | 機能 |
|------|------|
| `/` | 検索フォーカス |
| `f` | 画面にフィット |
| `r` | レイアウトリセット |
| `e` | PNG エクスポート |
| `Esc` | 選択解除 |
| `?` | ヘルプ表示 |

### パネル機能

#### Hotspots（リファクタリング優先度）
- 問題の多いモジュールを優先度順に表示
- クリックで該当モジュールにジャンプ

#### Key Modules（重要モジュール）
- Connections: 接続数でソート
- Issues: 問題数でソート
- Health: 健全性スコアでソート

#### Analysis（影響分析）
- Show Dependents: このモジュールに依存するモジュール
- Show Dependencies: このモジュールが依存するモジュール
- Full Impact: 全影響範囲（Blast Radius）

#### Filters（フィルタリング）
- Strength: Intrusive/Functional/Model/Contract
- Distance: SameFunction/SameModule/DifferentModule/DifferentCrate
- Volatility: High/Medium/Low
- Balance Score: 範囲指定
- Show Issues Only: 問題のあるエッジのみ
- Show Cycles Only: 循環依存のみ

### エクスポート
- **PNG Image**: グラフを画像として保存
- **JSON Data**: 分析データをJSON形式で保存

## 出力例

```
Analyzing project at './src'...
Analysis complete: 16 files, 16 modules

Starting web server at http://localhost:3000
Opening browser...
Press Ctrl+C to stop the server
```

## 注意事項

- Ctrl+C でサーバーを停止
- 大規模プロジェクトでは初回読み込みに時間がかかる場合があります
- ソースコード表示機能は分析対象のファイルのみ対応
