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

### 2.1–2.4 コア純粋ロジック（まとめて実施）
- 4 タスクとも OS 非依存で、Mac の単体テストだけで完結した。順序は **2.1 → 2.4 → 2.2 → 2.3**（不可視文字の分類表を Rust と TS で共有するため、2.1 の直後に 2.4）
- 分類表は 1.4 と同じ方式で `tests/fixtures/charset.json` に置き、Rust（`analysis/charset.rs`）と TS（`preview/charset.ts`）の両方のテストが同じ fixture を読む
- 機種依存文字は `encoding_rs` の SHIFT_JIS エンコード結果が CP932 拡張範囲かで判定。JIS 標準と NEC 行 13 の両方にある「≒」は標準側に符号化されるため非該当になる（意図どおり）
- `tauri init` が `Cargo.toml` の `tauri` 行を書き換えていたため、テキスト置換での依存追加が空振りした。**依存追加は `cargo add` を使う**
- 転送変換の適用順（7.12）は、各変換が「空白→空白または削除」のため出力から観測できない。テストは複合結果のみを検証し、順序は実装の構造で担保
- Rust テスト 33 件、Vitest 15 件。次の 3.x からクリップボード（OS 依存）に入る
- 2.1 のコミットは新規の JSON fixture に Prettier をかけ忘れて CI の lint が 3 OS で落ちた（次のコミットで解消）。**タスクの境界に関係なく、コミット前に `npm run lint` を必ず通す**

### 3.1–3.2 クリップボード層
- OS 依存コードに入る前に **スパイク（使い捨ての `examples/`）で `clipboard-rs` の実挙動を macOS で確認**した：自分の書き込みに独自形式（マーカー）を載せると `available_formats` に残る、`pbcopy` からのコピーは 200ms 間隔のポーリングで約 200ms で検知、秘匿形式名が `available_formats` に現れる。設計の前提がすべて成立したので本実装へ
- 実クリップボードを触るテストは `#[ignore]` にし、`cargo test -- --ignored` で macOS 上で手動実行する。CI では走らない（Linux ランナーにディスプレイがない）
- Windows / X11 のコードは Mac ではコンパイルされないため、**コミット → push → CI 3 OS 成功を確認してからタスク完了**にした（今回は一発で成功）
- 発見：`clipboard-rs` 0.3.5 は `wayland` feature で Wayland（data-control）にネイティブ対応し、`WAYLAND_DISPLAY` で実行時に切り替える。設計時の調査（X11 のみ）から更新されているため、3.3 の方針を見直す
