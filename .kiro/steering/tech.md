# Technology Stack

## Architecture

**Tauri 2** によるデスクトップアプリ。Rust のコアプロセスが OS のクリップボード・ウィンドウ・トレイを扱い、
WebView 上のフロントエンドがリスト表示と操作 UI を担う。両者は Tauri の command（frontend → Rust）と
event（Rust → frontend）で通信する。

```
OS clipboard ──(監視)──> Rust core ──(event: 新項目)──> WebView UI
                          ▲                               │
                          └──(command: 転送/削除/設定)────┘
```

責務の分割：
- **Rust core**: クリップボードの監視・読み書き（OS 別実装）、警告判定、テキスト変換、リストの保持、設定の永続化
- **Frontend**: 1 行プレビューの描画（不可視文字の記号化）、警告アイコン、トグル、キーボード操作、i18n
- 判定・変換ロジックは Rust 側に置き、UI は結果を描画するだけにする（3 OS で同一結果を保証し、単体テストを Rust に集約する）

## Core Technologies

- **Core**: Rust（stable、edition 2024）+ Tauri 2
- **Frontend**: TypeScript + Svelte 5 + Vite（小さな常駐 UI に対して最小のボイラープレートとバンドルサイズ）
- **Package manager**: npm（Node 26 以上）
- **Settings persistence**: アプリのデータディレクトリの `settings.json`（`serde_json`、原子的書き込み。Rust からのみアクセス）

## Key Libraries

Tauri 公式プラグインを優先し、3 OS 対応を自前で書かない：
- `tauri-plugin-global-shortcut` — グローバルホットキー
- `tauri-plugin-autostart` — OS ログイン時の自動起動
- `tauri-plugin-updater` — GitHub Releases からの更新（minisign 署名）
- `tauri-plugin-single-instance` — 二重起動防止
- Tauri 本体の tray / always-on-top / window state

**クリップボードは公式プラグインを使わず `clipboard-rs`（`default-features = false`, `wayland` feature）を使う**。
理由：変化監視、HTML/RTF の生データ取得、任意形式（秘匿マーク・自己マーカー）の読み書きが公式プラグインの範囲外で、
`clipboard-rs` はこれらを 3 OS（Linux は X11 / Wayland）で提供するため。共通トレイト `ClipboardPort` の裏に置き、
テストにはフェイクを使う（詳細は structure.md）。

## Development Standards

### Rust
- `cargo clippy -- -D warnings` をクリーンに保つ。`cargo fmt` 準拠
- OS 依存コードは `#[cfg(target_os = "...")]` でモジュール単位に分離し、関数内 `cfg` の散在を避ける
- `unsafe` は OS API 呼び出しモジュールに閉じ込め、安全なラッパーだけを公開する
- エラーは `thiserror` で型付けし、UI に出す文言は frontend 側で i18n する（Rust はエラー種別を返す）

### TypeScript / Svelte
- `strict: true`、`any` 禁止
- ESLint + Prettier
- UI 文言はすべて i18n リソース（`ja` / `en`）経由。ハードコード禁止

### Testing
- **Rust 単体テスト**（`#[cfg(test)]`）: 警告判定・テキスト変換・リスト管理は入力→出力の表で網羅する。3 OS の CI で実行
- **Frontend**: Vitest でプレビュー描画（不可視文字の記号化）を検証
- **E2E**: `tauri-driver` 単体は Windows / Linux のみ対応だが、`@wdio/tauri-service`（アプリ内蔵 WebDriver）は macOS を含む 3 OS で動作する。導入する場合はこちらを使い、CI では Linux ジョブで実行する
- 取り込んだテキストをテストログに出力しない

### Privacy
- 取り込んだ項目はメモリ上のみ。ディスク・ログ・ネットワークに出さない
- 設定ファイルには項目内容を含めない

## Development Environment

### Required Tools
- Rust stable（rustup）、Node 26+、Tauri CLI（`npm run tauri`）
- macOS: Xcode Command Line Tools
- Windows / Linux のビルドは GitHub Actions に任せる（Tauri はクロスコンパイル不可）

### Common Commands
```bash
npm run tauri dev          # 開発起動（Mac）
npm run tauri build        # 配布物ビルド（実行 OS 向け）
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
npm test                   # Vitest
```

### Platform Workflow
- **日常開発と検証は macOS** で行う
- **CI（GitHub Actions）** が push ごとに 3 OS でビルドと `cargo test` を実行し、`cfg(windows)` / `cfg(linux)` 配下のコンパイルエラーを検出する
- **Windows / Linux の手動確認は節目のみ**（VM 上で CI 成果物を実行）：クリップボード監視の完成時、ウィンドウ/トレイ/ホットキーの完成時、リリース前
- Linux は X11 と Wayland（KDE または wlroots 系）の両方を確認する。GNOME Wayland は非対応と明記

## Key Technical Decisions

- **Tauri 2 を採用**: 不可視文字の可視化は HTML/CSS が最も表現しやすい。バイナリが小さい。3 OS のビルド・macOS 公証・インストーラ生成が公式 CI ワークフローに揃う。依存が MIT/Apache-2.0 で再配布に障壁がない
- **判定・変換は Rust 側**: OS 間で結果を揃え、テストを 1 か所に集める
- **クリップボードは `clipboard-rs`（`wayland` feature 有効）**: 公式 Tauri プラグインは変化監視・RTF・任意形式に対応しないため。監視方式は Windows = クリップボードリスナー、macOS = changeCount のポーリング、Linux = X11 XFixes または Wayland data-control のポーリング（`WAYLAND_DISPLAY` で実行時選択。GNOME は data-control 非対応のため XWayland 経由の限定動作）
- **配布**: GitHub Releases + `tauri-action`。macOS は Developer ID 署名 + 公証、Windows は未署名（SmartScreen 警告は README で案内、費用をかけない）、Linux は AppImage / .deb / .rpm。Windows インストーラは NSIS を既定とし、MSI は winget 対応時に追加
- **更新は承諾制**: 起動時に確認・通知し、ユーザーが承諾したときだけ適用する
- **ライセンス**: アプリは MIT。依存の THIRD-PARTY 一覧を `cargo-about` 等でビルド時に生成し同梱する

---
_Document standards and patterns, not every dependency_
