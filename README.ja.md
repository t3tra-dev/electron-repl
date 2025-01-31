[[英語/English](README.md)]

# electron-repl

ElectronアプリケーションのメインプロセスでJavaScriptコードを実行できるコマンドラインREPLツールです。

## 特徴

- 実行中のElectronアプリケーションに接続
- メインプロセスでのJavaScriptコード実行
- macOSとLinuxをサポート（Windows対応は近日公開）
- コマンド履歴機能
- 見やすい色付き出力

## インストール

```bash
cargo install electron-repl
```

## 使い方

```bash
electron-repl <アプリ名> [ポート番号]
```

### 引数

- `アプリ名`: Electronアプリケーションの名前 (必須)
- `ポート番号`: DevTools用のポート番号 (デフォルト: 9222)

### 使用例

```bash
electron-repl Discord
```

## 対応プラットフォーム

- macOS
- Linux
- Windows（近日対応予定）

## ライセンス

MIT License - 詳細は[LICENSE](LICENSE)をご覧ください
