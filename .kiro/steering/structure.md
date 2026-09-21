# Project Structure

## Organization Philosophy

Tauri の標準レイアウト（`src/` = frontend、`src-tauri/` = Rust core）を採り、Rust 側は
**「OS 依存」と「純粋ロジック」を明確に分ける層構造**にする。純粋ロジック（判定・変換・リスト）は
OS を一切参照せず単体テスト可能に保ち、OS 依存はクリップボードとプラットフォーム固有モジュールに閉じ込める。

## Directory Patterns

### Rust core
**Location**: `src-tauri/src/`
**Purpose**: アプリの本体ロジック。以下のモジュール分割を守る

- `clipboard/` — クリップボードの監視・読み書き。`mod.rs` に共通トレイト（監視開始、読み取り、書き込み、自己書き込みの識別）を定義し、`macos.rs` / `windows.rs` / `linux.rs` が実装する。OS API と `unsafe` はここにだけ置く
- `analysis/` — 警告判定（スタイル有無、先頭末尾空白、タブ、機種依存文字、制御文字、改行コード混在、BOM/双方向制御/正規化混在）。純粋関数。1 警告 = 1 関数
- `transform/` — 転送時のテキスト変換（改行処理、タブ変換、全角空白変換、トリム）と適用順序。純粋関数
- `buffer/` — 取り込み項目のモデルと FIFO 保持（上限、連続重複排除、削除）
- `settings/` — 設定の型と永続化
- `commands/` — Tauri command の薄い入口。ロジックを持たず、上記モジュールへ委譲する
- `main.rs` / `lib.rs` — プラグイン登録、トレイ、ウィンドウ、イベント配線

**Example**: 新しい警告を追加する → `analysis/` に関数を 1 つ足し、警告の enum に variant を追加し、frontend のアイコンと i18n 文言を足す。他のモジュールは変えない

### Frontend
**Location**: `src/`
**Purpose**: 表示と操作のみ。判定・変換は行わない

- `lib/components/` — Svelte コンポーネント。`ItemRow`（1 行プレビュー + 警告列 + ホバーアクション）、`ItemList`、`TransferOptions`（トグル列）、`WarningIcon` など、画面上の単位で分ける
- `lib/preview/` — テキストを「表示用トークン列」に変換する純粋関数（空白・改行・不可視文字の記号化）。Vitest の対象
- `lib/ipc/` — Tauri command / event の呼び出しを型付きでラップする。コンポーネントから `invoke` を直接呼ばない
- `lib/stores/` — リスト、選択状態、トグル状態のストア
- `locales/` — `ja.json` / `en.json`。キーは `画面.要素` の階層で命名

### Platform verification and CI
**Location**: `.github/workflows/`、`doc/`
**Purpose**: `ci.yml`（push ごとに 3 OS でビルド + テスト）、`release.yml`（タグで配布物生成）。`doc/` には設計判断とプラットフォーム別の手動確認チェックリストを置く

### Specs and steering
**Location**: `.kiro/specs/`、`.kiro/steering/`
**Purpose**: 仕様（requirements / design / tasks）とプロジェクト記憶。コードを変える前に仕様を更新する

## Naming Conventions

- **Rust**: モジュール・関数・変数は `snake_case`、型・enum variant は `PascalCase`。警告種別は `Warning::HasStyle` のように「状態を表す名詞句」
- **Svelte components**: `PascalCase.svelte`。ファイル名 = コンポーネント名
- **TypeScript**: 関数・変数は `camelCase`、型は `PascalCase`。ストアは `xxxStore`
- **Tauri commands**: Rust 側 `snake_case`（例 `transfer_item`）。frontend の `lib/ipc/` で同名の `camelCase` 関数に包む
- **Events (Rust → frontend)**: `clipbuf://item-added` のようにプレフィックス付き kebab-case
- **i18n keys**: `list.warning.hasStyle` のようにドット区切りの階層

## Import Organization

```typescript
// frontend: $lib エイリアス（SvelteKit 相当の設定を Vite に置く）
import ItemRow from '$lib/components/ItemRow.svelte'
import { tokenize } from '$lib/preview/tokenize'
import { transferItem } from '$lib/ipc/commands'
```

```rust
// Rust: クレート内は絶対パス
use crate::analysis::Warning;
use crate::transform::apply;
```

## Code Organization Principles

- **依存の向き**: `commands` → `buffer` / `analysis` / `transform` / `settings` / `clipboard`。純粋ロジックのモジュール（`analysis`, `transform`, `buffer`）は `clipboard` や Tauri に依存しない
- **OS 分岐はモジュール境界で**: `#[cfg(target_os)]` は `clipboard/mod.rs` のモジュール選択と、プラットフォーム固有の初期化にだけ現れる
- **データは Rust が正**: リストの状態は Rust 側が保持し、frontend はイベントで受け取った写しを描画する。frontend から直接リストを変更しない
- **項目の内容を外に出さない**: ログ、設定ファイル、エラーメッセージに項目テキストを含めない
- **非破壊**: `transform` は新しい文字列を返す。`buffer` の項目は変更しない

---
_Document patterns, not file trees. New files following patterns shouldn't require updates_
