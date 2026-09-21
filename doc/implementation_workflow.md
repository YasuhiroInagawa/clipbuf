# 実装の進め方メモ（kiro-impl 以降）

マルチプラットホーム（Windows / macOS / Linux）のデスクトップアプリを **macOS 1 台で開発する**ための手順の概要。
仕様は `.kiro/specs/clipbuf-mvp/`、タスク一覧は同ディレクトリの `tasks.md`。

## 1. 1 タスクの進め方（`/kiro-impl clipbuf-mvp <番号>`）

1. **Task Brief** — `requirements.md` / `design.md` の該当節を読み、受け入れ基準・完了定義・設計上の制約・検証方法を整理する
2. **TDD** — 行動を伴うタスクは RED（失敗するテスト）→ GREEN → REFACTOR。雛形・設定だけのタスクは省略
3. **検証コマンド**（全タスク共通）
   ```bash
   npm run lint          # ESLint + Prettier
   npm run check         # svelte-check
   npm test              # Vitest
   npm run build         # フロントエンドのビルド（cargo build の前提）
   cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo build
   ```
4. **レビュー**（`kiro-review` プロトコル）— 差分・テスト・境界・要件/設計との整合を確認し APPROVED になるまで直す
5. **完了** — `tasks.md` を `[x]` にし、変更ファイルだけを明示的に `git add` してコミット（`feat(clipbuf-mvp): ... (task X.Y)`）。push すると CI が 3 OS で走る

## 2. プラットフォームごとの検証の分担

| 対象 | どこで確認するか | いつ |
|---|---|---|
| OS 非依存ロジック（判定・変換・バッファ・UI） | macOS ローカルの単体テストと手動確認 | 毎タスク |
| macOS 固有（クリップボード、トレイ、ホットキー） | macOS ローカルで実行 | 該当タスク |
| Windows / Linux 固有コード | **GitHub Actions の該当 OS でビルド・テストが通ること**を完了条件にする（Tauri はクロスコンパイル不可） | 毎 push |
| Windows / Linux の実動作 | VM（UTM / Parallels）に CI 成果物を入れて `doc/platform-checklist.md` で手動確認 | 節目のみ：クリップボード監視完成時、ウィンドウ/トレイ/ホットキー完成時、リリース前 |
| Linux の X11 と Wayland | 同じ VM でセッションを切り替えて両方確認（GNOME Wayland は非対応の通知が出ることを確認） | 節目のみ |

CI の 3 ジョブは `.github/workflows/ci.yml`。`#[cfg(target_os = "...")]` 配下のコードは macOS ではコンパイルされないため、**push して CI を見るまで Windows / Linux のコンパイルエラーは分からない**。OS 固有タスクでは小さく push する。

## 3. 開発環境の前提

- Node は nodebrew（`~/.nodebrew/current/bin`）、Rust は WorkData 側（`RUSTUP_HOME` / `CARGO_HOME` / `CARGO_TARGET_DIR` を `~/.zshrc` で指定）。ビルド成果物は `/Volumes/WorkData/Users/iyasuh/.cargo-target` に集約され、システムディスクを使わない
- `npm run tauri dev` で開発起動。初回はフルビルドで数分、以降は差分ビルド

## 4. タスクごとの記録

### 1.2 雛形
- `create-tauri-app` の svelte-ts テンプレートは SvelteKit 前提だったため、設計（`src/main.ts` + `App.svelte`）に合わせて **Vite の svelte-ts テンプレート + `tauri init`** の組み合わせにした
- ウィンドウ属性（常時最前面など）は `src-tauri/tauri.conf.json` に集約。macOS でアクセシビリティ権限がないと AppleScript でのウィンドウ属性取得はできないので、目視確認で代替

### 1.3 CI
- Windows ランナーは git が LF→CRLF 変換するため Prettier（`endOfLine: lf`）が全ファイルで失敗した。**`.gitattributes` で `* text=auto eol=lf`** を指定して解決
- Linux ランナーには Tauri の依存（webkit2gtk-4.1, gtk3, ayatana-appindicator3, librsvg2）に加え、後のクリップボード実装（x11rb / XFixes）で必要になる `libxcb-*-dev` を先に入れてある
- `cargo build` の前に `npm run build` を走らせ、`frontendDist`（`dist/`）が存在する状態にしている

### 1.4 共有型契約
- Rust の `model` と TS の `src/lib/ipc/types.ts` を **1 つの fixture JSON（`tests/fixtures/contract.json`）** で相互に固定する方式にした。Rust 側はデシリアライズ→再シリアライズが fixture と一致すること、TS 側は enum 定数配列と key 集合が fixture と一致することを検証する。片側だけ変えると両方のテストが落ちる（変異テストで確認済み）
- `model` は他層を import しない。テストであっても上向き依存（`crate::app`）は置かず、イベント名の検証は `app/events.rs` 側に置いた
- frontend へ渡す `CaptureCapability` / `PlatformInfo` は設計上 `clipboard` 寄りだが、依存の向き（`clipboard` → `model`）を守るため `model` に配置
