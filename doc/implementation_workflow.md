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

### 3.3 Wayland（設計変更）
- 設計時の調査が古かった例。実装フェーズで依存クレートのソースを読んで判明したため、**design.md / research.md / tasks.md / steering を先に更新してコミットし、それから実装**した（仕様と実装のずれを残さない）
- 自前 `WaylandAdapter` は不要になり、`clipboard-rs` の `wayland` feature 有効化と `capability_for`（純粋関数）の追加だけで完了。Linux 固有のバックエンド判定は `cfg(target_os = "linux")` の 1 関数に閉じた
- Linux ターゲット限定の依存（`wl-clipboard-rs`）は macOS / Windows のビルドに影響しない。CI 3 OS で確認

### 3.4 アダプタ選択・macOS 許可・プラットフォーム情報
- `cfg(target_os)` で分岐するコードは、**Mac でクリーンでも他 OS では `unused mut` / 未使用 import で clippy が落ちる**ことがある。対策：(a) cfg 分岐は関数ごと切り替える（`#[cfg] fn f(mut x)` / `#[cfg(not)] fn f(x)`）、(b) 各 OS 用のテストを用意して import を全 OS で使う
- CI の 3 OS それぞれで `select_adapter` の実環境テストが走る：macOS = Full、Windows = Full（ランナーに実クリップボードあり）、Linux = Unavailable（ヘッドレス）。これで「ディスプレイなしでも落ちない」経路が毎回検証される
- macOS の `accessBehavior`（15.4+）は `respondsToSelector` でガードし、古い macOS でのクラッシュを防ぐ。`objc2-app-kit` は `[target.'cfg(target_os = "macos")'.dependencies]` に置き、`NSPasteboard` feature だけ有効化

### 4.1 設定ストア（設計変更）
- `tauri-plugin-store` をやめて素の JSON ファイルにした。理由：設定は Rust からしか触らない、tempdir で単体テストが完結する、プラグインの最新公開版が alpha。Tauri 依存は `load(&AppHandle)` のパス解決 1 行に閉じ込め、ロジック（validate / sanitize / diff）は純粋関数
- 破損ファイル・部分ファイル・型違いの値は**フィールド単位で既定値に戻す**（ファイル全体を捨てない）。不正な更新は書き込み前に拒否
- 書き込みは一時ファイル → rename の原子的更新。ディレクトリ作成失敗は `SettingsIo` で返し、パニックさせない

### 4.2 取り込みサービス
- Tauri の `emit` を `CaptureSink` トレイトで切り離し、サービス本体は Tauri に依存しない。テストは記録用シンク + `FakeClipboard` で、スレッドを使う統合テストも `mpsc` の `recv_timeout` で決定的に書ける
- ログには項目内容を絶対に出さない。除外理由も `Debug` ではなく静的文字列で出す（`Decision::reason()`）。レビューで見つけた構造的リスクをその場で潰した例
- デバウンスは「静かになるまで待つ」方式に上限（500ms）を付け、連続書き込みでも 1 秒以内の要件を保つ

### 4.3 コマンドとイベント
- Tauri の `#[tauri::command]` は**関数名がそのままフロントのコマンド名**になるため、ロジック関数と同名にできない。ロジックを `app/ops.rs`（Tauri 非依存・テスト対象）に置き、`app/commands.rs` は同名の薄いラッパーだけにした
- フロントへのイベント送出は `EventSink` トレイト経由。テストでは記録用シンク、本番は `TauriSink(AppHandle)`。これでコマンドのロジックも `FakeClipboard` だけで統合テストできる
- `bootstrap` を実装したことで `npm run tauri dev` の起動時に設定ファイルが生成され、取り込みスレッドが走る状態になった。UI がまだ無いので、起動と設定ファイル生成をスモークテストとした

### 4.4 ウィンドウ・トレイ・ホットキー
- ホットキーの「差し替え失敗時は旧を復元」規則は、プラグインを `Registrar` トレイトで抽象化してフェイクで単体テストした。`Shortcut` の `Display` は修飾キー順を正規化する（`alt+shift` → `shift+alt`）ので、比較は解析済みの値で行う
- **実機確認が必要な項目（キー押下、トレイクリック）は AppleScript の権限がなく自動化できないため、ユーザーに手順を渡して確認してもらった**。その際、**私がバックグラウンドで起動した開発インスタンスがポート 1420 を掴んだままでユーザーの `tauri dev` が失敗した**。以後、確認を依頼する前に自分の起動分は必ず止める
- ユーザー確認から 2 点の UX 改善を取り込んだ：Settings ウィンドウの配置（メインの横）、メインと Settings の表示・非表示同期。仕様に無かった振る舞いは design.md に追記してから実装
- `env_logger` を `debug_assertions` 時のみ初期化。`RUST_LOG=clipbuf_lib=debug npm run tauri dev` でモジュールのログが見える

### 4.5 常駐と設定差分の適用
- 設定更新は `apply_settings` 1 本に結合：validate → **ホットキー差分は保存前に登録**（失敗なら何も変えない）→ 保存 → 上限切り詰め / 自動起動 → イベント。プラグインは `Registrar` / `AutostartControl` トレイトで抽象化し、7 件のフェイクテストで固定
- 単一インスタンスはデバッグバイナリを直接 2 本起動して確認できる（2 つ目が即終了し 1 つ目が残る）。バイナリはデバッグビルドでも devUrl を見に行くだけなので、dev サーバー無しでプロセスの起動確認はできる
- バンドルされていないデバッグバイナリには AppleEvent の quit が届かないため、終了時保存の経路は手動確認（トレイの Quit）に頼る。非表示時にも位置を保存するようにして依存を減らした
- 開発起動でバイナリに引数を渡すには `npm run tauri dev -- -- -- --hidden`（`--` が 3 つ：npm / Tauri CLI / cargo がそれぞれ 1 つ消費）
- メインウィンドウは `visible: false` で生成し、`bootstrap` で表示判定する（`--hidden` と起動時のちらつき対策を兼ねる）

### 5.1 frontend の IPC・ストア・i18n
- ストアは `svelte/store` で書き、Tauri の `invoke` / `listen` は**インターフェース（`ItemsApi` 等）で注入**する。これで Vitest（node 環境）だけでストアの挙動を検証でき、Tauri の実行環境は不要
- 「コンポーネントは `lib/ipc` 以外から Tauri を import しない」という設計制約は、**ESLint の `no-restricted-imports`** で機械的に担保した（レビューの目視に頼らない）
- 文言は後続タスクの分まで先に ja/en に揃え、キー集合の一致をテストで固定。文言追加時は両方に入れないとテストが落ちる
- `.kiro/specs/` 配下に空ファイル（`clipbuf@0.1.0`, `tauri`）が生成されていた。npm の出力をシェルに貼り付けたときのリダイレクト事故と思われる。削除した

### 5.2 最初の Svelte コンポーネント
- コンポーネントテストは `@testing-library/svelte` + `jsdom`。Vitest の既定環境は node のまま、コンポーネントテストだけ先頭の `/** @vitest-environment jsdom */` で切り替える（純粋モジュールのテストを速いままにする）
- `vite.config.ts` に `svelteTesting()` プラグイン、`src/test-setup.ts` に jest-dom のマッチャを追加
- i18n はモジュールシングルトン（`t` ストア）をコンポーネントが直接 import。テストでは `i18n.setLanguage('en')` で固定
- 表示の目視確認はリストを組み立てる 5.4 でまとめて行う（部品単体は DOM テストで担保）

### 5.4 項目リストとメインウィンドウ（実装済み・手動確認は途中）
- `MainWindow` は `MainContext`（stores + `MainApi`）を 1 つの props で受ける。テストはフェイクの `MainApi` で 13 件（描画、クリック転送とハイライト、ホバーボタン、通知、キー操作、バナー、`window-shown`、全削除）
- 実画面の確認用にテストデータを `pbcopy` で投入するときは **`LANG=ja_JP.UTF-8` を付ける**。ツールのシェルはロケール未設定で、非 ASCII が化けた（「①」が「竭」になった）
- `screencapture` は「画面収録」権限が必要で、ユーザーに権限要求ダイアログが出てしまう。**使わない**。画面の確認はユーザーに依頼するか、スクリーンショットを貼ってもらう
- ホバー時の代替転送ボタンの表示（暫定の「T」「=」）は分かりにくいとの指摘あり。ラベル/記号の案を出して未決

## 次回の再開手順（5.4 の手動確認から）

1. アプリ起動（ユーザー側のターミナルで）
   ```bash
   npm run tauri dev
   ```
2. テストデータ投入（別ターミナル。UTF-8 ロケール必須）
   ```bash
   export LANG=ja_JP.UTF-8
   printf 'hello\tworld  \n' | pbcopy; sleep 1
   printf 'カタカナ　全角スペース\r\nCRLF行\n2行目' | pbcopy; sleep 1
   printf '  leading spaces and ① NEC char' | pbcopy; sleep 1
   printf 'zero\xe2\x80\x8bwidth and nbsp\xc2\xa0here' | pbcopy
   ```
3. 確認項目
   - 記号（→ · ↵ □ ∅ ⍽）と警告アイコン（⎵ ⇥ ㊙ ⏎、ホバーで説明）
   - 行クリック → 緑のハイライト → 他アプリに貼り付けできる。順序と選択が変わらない
   - ホバーの「T」（プレーン）「=」（元のまま）ボタン ← 表示の改善案を決める
   - ↑↓ / Enter / Shift+Enter / Delete / Escape
   - トグル変更（例：改行 → 空白）が転送に反映され、再起動後も残る
   - 「すべて削除」と空表示
4. 確認できたら `tasks.md` の 5.4 を `[x]` にしてコミット → 5.5（設定ウィンドウ）へ
