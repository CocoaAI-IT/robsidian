# Robsidian

Rust製のObsidian風マークダウンノートアプリケーションです。

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows_11-0078D6?style=flat&logo=windows&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green.svg)

## 機能

- **マークダウン編集**: シンタックスハイライト付きのテキストエディタ
- **ライブプレビュー**: リアルタイムでマークダウンをレンダリング
- **分割ビュー**: エディタとプレビューを同時に表示
- **ファイルエクスプローラー**: Vault（ワークスペース）内のファイルをツリー表示
- **ターミナル**: アプリ内でコマンドを実行可能
- **プラグインシステム**: WASM形式のプラグインに対応（開発中）

## 必要な環境

- Windows 11 (64bit)
- インターネット接続（初回セットアップ時）

> **Note**: Visual Studio や Visual Studio Build Tools は**不要**です。
> このプロジェクトはMinGW-w64を使用してビルドします。

---

## セットアップ手順（初心者向け）

### Step 1: Scoopのインストール

Scoopは、Windowsでソフトウェアを簡単にインストールできるパッケージマネージャーです。

1. **PowerShellを管理者として開く**
   - スタートメニューで「PowerShell」と検索
   - 「Windows PowerShell」を右クリック →「管理者として実行」

2. **以下のコマンドを順番に実行**

   ```powershell
   # 実行ポリシーを変更（Scoopのインストールに必要）
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

   # Scoopをインストール
   Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
   ```

3. **インストール確認**
   ```powershell
   scoop --version
   ```
   バージョン番号が表示されればOKです。

---

### Step 2: MinGW-w64のインストール

MinGW-w64は、Windows用のGCCコンパイラツールチェーンです。

```powershell
scoop install mingw
```

インストール後、以下のコマンドで確認：
```powershell
gcc --version
```

`gcc (x86_64-posix-seh-rev0, Built by MinGW-Builds project)` のような表示が出ればOKです。

---

### Step 3: Rustのインストール

1. **rustupをダウンロード**

   以下のURLからインストーラをダウンロードして実行：
   https://rustup.rs/

   または、PowerShellで以下を実行：
   ```powershell
   Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
   .\rustup-init.exe
   ```

2. **インストールオプションの選択**

   インストーラが起動したら、以下のように選択します：

   ```
   1) Proceed with standard installation (default - just press enter)
   2) Customize installation
   3) Cancel installation

   > 2  ← カスタムインストールを選択
   ```

   カスタム設定画面で：
   ```
   Default host triple? [x86_64-pc-windows-msvc]
   > x86_64-pc-windows-gnu  ← これを入力（重要！）

   Default toolchain? [stable]
   > stable  ← そのままEnter

   Profile? [default]
   > default  ← そのままEnter

   Modify PATH variable? [Y/n]
   > Y  ← そのままEnter
   ```

3. **新しいターミナルを開いて確認**
   ```powershell
   rustc --version
   cargo --version
   ```

   以下のような出力が表示されればOKです：
   ```
   rustc 1.XX.X (xxxxxxx 202X-XX-XX)
   cargo 1.XX.X (xxxxxxx 202X-XX-XX)
   ```

---

### Step 4: Gitのインストール（オプション）

ソースコードをダウンロードするために必要です。

```powershell
scoop install git
```

---

### Step 5: ソースコードの取得

1. **Gitでクローン**
   ```powershell
   git clone https://github.com/CocoaAI-IT/rbsidian.git
   cd rbsidian
   ```

   **または**、GitHubから直接ZIPをダウンロード：
   - https://github.com/CocoaAI-IT/rbsidian にアクセス
   - 緑色の「Code」ボタン →「Download ZIP」
   - 展開して、そのフォルダに移動

---

### Step 6: ビルドと実行

1. **プロジェクトフォルダに移動**
   ```powershell
   cd rbsidian
   ```

2. **ビルド（初回は時間がかかります）**
   ```powershell
   cargo build --release
   ```

   初回ビルドは依存関係のダウンロードとコンパイルのため、5〜10分程度かかる場合があります。

3. **実行**
   ```powershell
   cargo run --release
   ```

   または、直接実行ファイルを起動：
   ```powershell
   .\target\x86_64-pc-windows-gnu\release\robsidian.exe
   ```

---

## 使い方

### 基本操作

1. **Vaultを開く**
   - メニューバーの「File」→「Open Vault...」をクリック
   - マークダウンファイルが入っているフォルダを選択

2. **ファイルを開く**
   - 左側のファイルツリーからファイルをクリック

3. **編集とプレビュー**
   - 左側でマークダウンを編集
   - 右側にリアルタイムでプレビューが表示されます

### キーボードショートカット

| ショートカット | 機能 |
|--------------|------|
| `Ctrl + S` | ファイルを保存 |
| `Ctrl + B` | サイドバーの表示/非表示 |
| `Ctrl + `` ` | ターミナルの表示/非表示 |

### 表示モードの切り替え

メニューバーの「View」から選択できます：

- **Editor Only**: エディタのみ表示
- **Preview Only**: プレビューのみ表示
- **Split View**: エディタとプレビューを並べて表示（デフォルト）

---

## トラブルシューティング

### Q: `cargo build` で `dlltool.exe` が見つからないエラー

MinGWが正しくインストールされていないか、PATHに追加されていません。

```powershell
# MinGWを再インストール
scoop uninstall mingw
scoop install mingw

# 新しいターミナルを開いて再実行
cargo build --release
```

### Q: `link.exe` 関連のエラーが出る

Rustのホストターゲットが間違っている可能性があります。

```powershell
# 現在の設定を確認
rustup show

# GNUターゲットをデフォルトに設定
rustup default stable-x86_64-pc-windows-gnu
```

### Q: ビルドは成功するが起動時にDLLエラー

MinGWのDLLにパスが通っていない可能性があります。

```powershell
# 環境変数を確認
$env:Path -split ';' | Select-String mingw
```

表示されない場合は、MinGWのbinディレクトリをPATHに追加してください。

### Q: 日本語が文字化けする

現在のバージョンではシステムフォントを使用しています。
日本語フォントを追加する場合は、`assets/fonts/` にフォントファイルを配置し、
`src/app.rs` の `configure_fonts` 関数を編集してください。

---

## 開発者向け情報

### プロジェクト構造

```
robsidian/
├── .cargo/
│   └── config.toml      # ビルド設定（MinGW使用）
├── src/
│   ├── main.rs          # エントリーポイント
│   ├── app.rs           # アプリケーション状態管理
│   ├── core/            # コア機能（ドキュメント、ファイル操作、設定）
│   ├── ui/              # UIコンポーネント
│   ├── terminal/        # ターミナル機能
│   └── plugin/          # プラグインシステム
├── assets/              # 静的アセット
├── plugins/             # プラグインサンプル
├── Cargo.toml           # 依存関係
└── README.md
```

### 使用技術

| 用途 | クレート | バージョン |
|-----|---------|-----------|
| GUI フレームワーク | eframe / egui | 0.32 |
| マークダウンレンダリング | egui_commonmark | 0.21 |
| マークダウンパース | pulldown-cmark | 0.12 |
| ファイル監視 | notify | 8 |
| WASMランタイム | wasmtime | 28 |
| ファイルダイアログ | rfd | 0.15 |

### デバッグビルド

```powershell
cargo build
cargo run
```

### リリースビルド

```powershell
cargo build --release
```

---

## ライセンス

MIT License

---

## 貢献

Issue や Pull Request を歓迎します！

1. このリポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add amazing feature'`)
4. ブランチをプッシュ (`git push origin feature/amazing-feature`)
5. Pull Request を作成

---

## 謝辞

- [egui](https://github.com/emilk/egui) - 素晴らしいRust GUIライブラリ
- [Obsidian](https://obsidian.md/) - インスピレーションの源
