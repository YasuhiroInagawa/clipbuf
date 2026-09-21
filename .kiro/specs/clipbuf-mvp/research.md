# Research & Design Decisions

## Summary
- **Feature**: `clipbuf-mvp`
- **Discovery Scope**: New Feature（グリーンフィールド、フル調査）
- **Key Findings**:
  - クリップボードの「監視 + HTML/RTF の生データ取得 + 任意フォーマットの読み書き」は `clipboard-rs` が 3 OS（Linux は X11 のみ）で提供しており、自前で OS API を叩く範囲を Wayland のみに縮小できる
  - macOS 26 のペーストボードプライバシーは `changeCount` の監視では発火せず、内容の読み取りで発火する。ユーザーは「プライバシーとセキュリティ」でアプリごとに常に許可を設定できる。設計上は「監視はポーリング、読み取りは変化時のみ」で最小化し、拒否時は設定への案内を出す
  - Wayland ネイティブの監視は `wl-clipboard-watch`（ext-data-control / wlr-data-control）で可能。GNOME はどちらのプロトコルも実装しないため非対応のまま
  - E2E は `tauri-driver` 単体では macOS 非対応だが、`@wdio/tauri-service` はアプリ内に WebDriver サーバを埋め込む方式で 3 OS に対応する（steering の記述を更新）

## Research Log

### クリップボードアクセスライブラリの選定
- **Context**: 要件 1.1–1.5（監視、書式付きデータの保持、秘匿マーク検出、自己書き込み除外）と 6.x（HTML/RTF を含む書き戻し）を満たす手段
- **Sources Consulted**:
  - [ChurchTao/clipboard-rs](https://github.com/ChurchTao/clipboard-rs) / [docs.rs](https://docs.rs/crate/clipboard-rs/latest)
  - [YaLTeR/wl-clipboard-rs](https://github.com/YaLTeR/wl-clipboard-rs)、[wl-clipboard-watch](https://docs.rs/wl-clipboard-watch/latest/wl_clipboard_watch/)
  - [Win32 Clipboard Formats](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats)
- **Findings**:
  - `clipboard-rs` 0.3 系: `get_text` / `get_html` / `get_rich_text`、`available_formats` による形式一覧、`get_buffer` / `set_buffer` による任意形式の読み書き、`ClipboardWatcher` + `ClipboardHandler` による変化通知。Windows・macOS・Linux(X11) 対応。Wayland は非対応
  - `wl-clipboard-rs` 0.9 系: ext-data-control を優先し wlr-data-control にフォールバック。`wl-clipboard-watch` が同プロトコルで selection イベントを受信し MIME 指定で内容を取得できる
  - 秘匿マーク: macOS は `org.nspasteboard.ConcealedType`、Windows は登録形式名 `ExcludeClipboardContentFromMonitorProcessing`、KDE は `x-kde-passwordManagerHint`。いずれも「形式名の存在」で判定でき、`available_formats` で取得できる見込み。Windows で登録形式の名前が `available_formats` に含まれるかは実装時に確認する
  - 自己書き込みの識別: 書き込み時に独自形式（例: `org.clipbuf.marker`）を同時に載せ、監視側で形式の存在を見て除外する。形式が落ちる環境に備え、直前に書いた内容のハッシュ比較を第 2 の判定にする
- **Implications**: OS 依存コードは「clipboard-rs アダプタ」と「Wayland アダプタ」の 2 実装に収まる。`unsafe` は不要。macOS の `accessBehavior` 判定のみ `objc2-app-kit` を直接使う

### macOS ペーストボードプライバシー
- **Context**: 要件 11.6（読み取り拒否時の案内）、1.8（1 秒以内の反映）
- **Sources Consulted**:
  - [Michael Tsai — Pasteboard Privacy Preview in macOS 15.4](https://mjtsai.com/blog/2025/05/12/pasteboard-privacy-preview-in-macos-15-4/)
  - [MacRumors — Apple to Block Mac Apps From Secretly Accessing Your Clipboard](https://www.macrumors.com/2025/05/12/apple-mac-apps-clipboard-change/)
  - [feedback-assistant #655 — NSPasteboard accessBehavior](https://github.com/feedback-assistant/reports/issues/655)
- **Findings**:
  - ユーザーのペースト操作を伴わない読み取りに対し、アプリごとの許可ダイアログが出る。設定は「常に許可 / 確認 / 拒否」の 3 段階
  - `changeCount` の参照は発火しない。`detectPatterns(for:)` も発火しない。内容の読み取り（`string(forType:)` 等）で発火する
  - `NSPasteboard.accessBehavior` で現在の許可状態を取得できる（macOS 15.4+）
- **Implications**: 監視は `changeCount` ポーリング（既定 200ms）、内容読み取りは変化時 1 回のみ。起動時に `accessBehavior` を確認し `deny` なら案内を表示。README に「常に許可」の設定手順を記載

### Linux Wayland の扱い
- **Context**: 要件 11.3–11.5
- **Findings**:
  - wlroots 系（sway, Hyprland 等）と KDE Plasma は data-control プロトコルを実装。GNOME（Mutter）は実装しない
  - XWayland 経由の X11 クリップボードは多くのコンポジタが Wayland 側と同期するため、X11 アダプタで Wayland アプリのコピーも観測できる場合がある（保証はない）
- **Implications**: 起動時に「data-control 利用可 → Wayland アダプタ」「X11 利用可 → X11 アダプタ（XWayland 含む）」「どちらも不可 → 取り込み不可を通知」の順で選択。XWayland フォールバック時は「取り込みが限定的」の通知を出す

### Tauri 2 プラグインの対応状況
- **Sources Consulted**: [Tauri Ecosystem Releases](https://v2.tauri.app/release/)、[Updater plugin](https://v2.tauri.app/plugin/updater/)、[Features & Recipes](https://v2.tauri.app/plugin/)
- **Findings**: global-shortcut 2.3.x、autostart 2.5.x、store 2.4.x、updater 2.10.x、single-instance / window-state も 2.x で安定。Rust と npm のバージョンは同期している。トレイと always-on-top は Tauri 本体の機能
- **Implications**: 要件 8.x / 9.x / 13.6–13.8 はすべて公式プラグインで充足。自前実装はクリップボードのみ

### E2E テスト基盤
- **Sources Consulted**: [Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/)、[WebdriverIO Tauri Service](https://webdriver.io/docs/wdio-tauri-service/)、[Platform Support](https://webdriver.io/docs/desktop-testing/tauri/platform-support/)
- **Findings**: `tauri-driver` 単体は Windows / Linux のみ。`@wdio/tauri-service` はアプリ内蔵の WebDriver サーバを使い、macOS を含む 3 OS で動作。IPC のモック、ログ取得も提供
- **Implications**: E2E を導入する場合は `@wdio/tauri-service` を採用し、CI の 3 OS で実行可能。steering `tech.md` の E2E 記述を更新した

### 機種依存文字の判定基準
- **Context**: 要件 5.4 の判定を明確化する
- **Findings**:
  - 日本語環境での「機種依存文字」は実務上 CP932（Windows-31J）のベンダー拡張を指す：NEC 特殊文字（Shift_JIS 0x8740–0x879E）、NEC 選定 IBM 拡張（0xED40–0xEEFC）、IBM 拡張（0xFA40–0xFC4B）
  - `encoding_rs` の SHIFT_JIS エンコーダは WHATWG 準拠で、NEC 選定 IBM 拡張の文字を IBM 拡張側（0xFAxx）にエンコードする。いずれの場合も上記の範囲に入るため判定に支障はない
  - JIS X 0208 に無い文字全般（é や絵文字）を機種依存扱いにすると欧文テキストで誤警告が多発する
- **Implications**: 判定は「Shift_JIS にエンコードした結果のバイト列が CP932 拡張範囲に入る文字が 1 つでもある」とする。エンコード不能な文字は対象外

### 制御文字・不正データの判定
- **Findings**: `clipboard-rs` は `String` を返すため、不正な UTF-8 は到達前に置換文字 U+FFFD になる。不正サロゲートは Rust の `String` には存在し得ない
- **Implications**: 要件 5.5 の判定は「タブ・LF・CR 以外の C0 制御文字、DEL、C1 制御文字（U+0080–U+009F）、または U+FFFD を含む」とする

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Rust core が状態を持ち、frontend は写しを描画（採用） | リスト・設定・判定・変換を Rust に集約し、event で frontend に通知 | 3 OS で同一結果、テスト集約、frontend が薄い | event と command の契約設計が必要 | steering `tech.md` の方針に一致 |
| frontend が状態を持ち Rust はクリップボード I/O のみ | JS 側でリストと判定を実装 | UI と状態が近く実装が速い | 判定ロジックが TS に分散し、CI の 3 OS テストから外れる | 却下 |
| ポート & アダプタ（clipboard を trait で抽象化） | `ClipboardPort` trait + OS 別アダプタ | Wayland / X11 / clipboard-rs を差し替え可能、テスト用フェイクが容易 | 実装が 2 つしかないため抽象が薄い | 採用（Wayland と clipboard-rs の 2 実装が実在するため indirection は正当） |

## Design Decisions

### Decision: クリップボードアクセスは `clipboard-rs` を採用し、Wayland のみ `wl-clipboard-*` で補完する
- **Context**: steering では「自前実装」としていたが、要件を満たす既存クレートが存在する
- **Alternatives Considered**:
  1. OS API を直接呼ぶ（windows-sys / objc2 / x11rb）— 制御は最大だが `unsafe` と保守量が増える
  2. `arboard` — 変化通知と RTF 取得が無く、任意形式の読み書きも限定的
  3. `clipboard-rs` + `wl-clipboard-watch` — 要件を満たし `unsafe` 不要
- **Selected Approach**: 3。`ClipboardPort` trait の裏に `ClipboardRsAdapter`（Win/mac/X11）と `WaylandAdapter` を置く
- **Rationale**: 監視・HTML/RTF・任意形式が揃い、OS 依存の自前コードを最小化できる
- **Trade-offs**: 個人メンテのクレートに依存する。フォーク可能な規模（数千行）であることを確認済み
- **Follow-up**: Windows で `available_formats` が登録形式名を返すこと、macOS で `set_buffer` の独自形式が保持されることを実装初期に確認する

### Decision: 自己書き込みの除外はマーカー形式 + 内容ハッシュの二重判定
- **Context**: 要件 1.4
- **Selected Approach**: 書き込み時に独自形式 `org.clipbuf.marker` を同時に載せる。監視側は (a) マーカー形式が存在する、または (b) テキストが直前の書き込み内容のハッシュと一致する、のいずれかで除外
- **Rationale**: マーカーは確実だが環境により形式が落ちる可能性がある。ハッシュは環境非依存だが同一内容を他アプリで再コピーした場合も除外してしまう（許容：連続重複排除 1.6 と同じ結果になる）

### Decision: 判定・変換・リストは Rust の純粋モジュールに置き、frontend はトークン化のみ持つ
- **Context**: 要件 3.x / 5.x / 7.x を 3 OS で同一にする
- **Selected Approach**: `analysis` / `transform` / `buffer` は OS にも Tauri にも依存しない。frontend の `preview/tokenize` は表示専用（記号化）で、判定結果は Rust から受け取る
- **Trade-offs**: 不可視文字の分類が Rust（警告）と TS（表示）に二重に存在する。分類対象の文字集合を design.md に一覧化し、双方のテストで同じ表を使う

### Decision: 設定画面は別ウィンドウ
- **Context**: 要件 8.1（小さなフローティング）と 9.1（設定画面）、9.6（転送トグルは設定画面に置かない）
- **Alternatives Considered**: メインウィンドウ内のパネル切替 / 別ウィンドウ
- **Selected Approach**: `settings` ラベルの別ウィンドウを遅延生成。メインウィンドウはリストとトグルのみ
- **Rationale**: 常時最前面の小ウィンドウに設定を押し込むとサイズ制約が厳しい

### Decision: 転送結果は command の戻り値で通知する
- **Context**: 要件 6.7 / 6.10 / 7.5
- **Selected Approach**: `transfer_item` が `TransferOutcome { skipped_transforms: bool }` を返し、失敗は `AppError` で返す。frontend がハイライトと通知を描画する
- **Rationale**: 転送は同期的で 1 回の往復で完結するため、event より戻り値が単純

### Decision: プレビューの描画上限
- **Context**: 要件 3.6 / 4.1 と大きなテキスト（数 MB）の両立
- **Selected Approach**: 警告判定は全文に対して行う。プレビュー行は先頭 20,000 文字までをトークン化し、超過分は省略記号で示す
- **Rationale**: 20,000 文字は横スクロールで閲覧する現実的な上限であり、DOM 負荷を抑えられる

### Decision: グローバルホットキーの既定値は `Alt+Shift+V`
- **Rationale**: `Ctrl/Cmd+Shift+V` は多くのアプリの「プレーンテキストで貼り付け」と衝突する。`Alt+Shift+V` は主要 OS・主要アプリの既定と衝突しにくい。設定で変更可能

### Decision: i18n は自前の最小実装
- **Context**: 2 言語・数十文言
- **Alternatives Considered**: `svelte-i18n` / 自前ストア
- **Selected Approach**: JSON リソースを読み込む `t(key)` ストア（数十行）。依存を増やさない
- **Follow-up**: 言語が 3 つ以上に増えたら `svelte-i18n` への移行を検討

## Risks & Mitigations
- `clipboard-rs` の Windows 実装で登録形式名が取れない → 実装初期に検証。取れなければ `windows-sys` で `EnumClipboardFormats` + `GetClipboardFormatNameW` を直接呼ぶ小さな補助関数を Windows アダプタに追加
- macOS 26 で読み取りのたびにダイアログが出る（「確認」設定） → 起動時に `accessBehavior` を確認し、`ask` の場合も案内を出す。README に「常に許可」の手順
- Wayland（GNOME）で取り込み不可 → 起動時通知と README で明記。XWayland フォールバックで部分対応
- 大量・高頻度のコピー（プログラムによる連続書き込み） → 監視コールバックはキューに積み、UI 更新は 100ms 単位でまとめる
- 個人メンテのクレート依存 → バージョン固定、必要ならフォーク

## References
- [ChurchTao/clipboard-rs](https://github.com/ChurchTao/clipboard-rs) — クリップボード監視・多形式アクセス
- [wl-clipboard-watch](https://docs.rs/wl-clipboard-watch/latest/wl_clipboard_watch/) — Wayland data-control による監視
- [Tauri Updater plugin](https://v2.tauri.app/plugin/updater/) — GitHub Releases からの更新
- [Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/) / [WebdriverIO Tauri Service](https://webdriver.io/docs/wdio-tauri-service/) — E2E
- [Pasteboard Privacy Preview in macOS 15.4](https://mjtsai.com/blog/2025/05/12/pasteboard-privacy-preview-in-macos-15-4/) — macOS の許可モデル
- [Win32 Clipboard Formats](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats) — 登録形式と秘匿マーク
