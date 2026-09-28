# Windows / Linux での検証引き継ぎ

macOS 以外の環境へ持っていって動作確認するための手引き。リポジトリに入っているので、
チェックアウトすればこの文書ごと付いてくる。

確認項目そのものは [platform-checklist.md](platform-checklist.md)、実装の経緯と落とし穴は
[implementation_workflow.md](implementation_workflow.md) にある。ここはその 2 つを使うための
段取りと、OS 別に必要な手元操作だけを書く。

## いまの状態

| | |
|---|---|
| 自動テスト | フロント 145 / Rust 119。CI は 3 OS すべて緑 |
| 配布物の生成 | 3 OS 分をリリースワークフローで生成確認済み（NSIS / AppImage / deb / rpm / dmg） |
| **実機での動作確認** | **macOS・Windows**（タスク 6.1）。**Linux は未確認** |

Linux は **一度も実機で動かしていない**。仕様上そこはチェックリスト運用なので
「仕様は満たしているが動作保証は macOS・Windows だけ」という状態。

Windows は 2026-09-27〜28 に実機（実クリップボード、NSIS インストーラ、アンインストーラ）で
[platform-checklist.md](platform-checklist.md) の Windows 項目を一通り確認済み。見つかった問題は
その場で修正しコミット済み（`publisher` 未設定で発行元が "gr" になっていた件、アプリアイコンが
Tauri のデフォルトのままだった件など）。SmartScreen の警告だけは、検証機に Defender を置き換える
サードパーティ製アンチウイルスが入っていたため確認できていない（[platform-checklist.md](platform-checklist.md)
に注記あり）。Windows のコード署名は取得しない方針なので、バージョン番号を `1.0.0` に上げる以外の
作業は完了している。

## どこが危ないか

自動テストは `FakeClipboard` を使う単体・結合テストなので、**OS 固有のクリップボード
アダプタは本物のクリップボード相手に一度も動いていない**。`src-tauri/src/clipboard/clipboard_rs.rs`
には単体テストが 1 つもない（純粋ロジックを外に出した設計の裏返し）。

したがって重点はここ。

**Windows**

- クリップボード変化の検出。Windows だけイベント駆動で、macOS / Wayland のポーリングとは別経路
- 秘匿マーカー `ExcludeClipboardContentFromMonitorProcessing` の検出
- 自己書き込み除外（マーカー形式を書けているか）
- トレイアイコン、グローバルホットキー
- NSIS インストーラが管理者権限なしで入るか、SmartScreen の挙動が README の記述と合っているか

**Linux**

- アダプタ選択が正しいか（X11 か Wayland の data-control か）。`設定` に「ポーリング間隔」が
  出るかどうかで判別できる（Wayland なら出る、X11 なら出ない）
- GNOME Wayland で「XWayland アプリからのみ取り込む」バナーが出るか（要件 11.5）
- トレイ。AppIndicator が無い環境ではアイコンが出ない。その場合はホットキーで操作する
- AppImage / deb / rpm それぞれの起動

## 環境構築

### 共通

Node 22 以上と Rust stable。リポジトリを clone して `npm ci`。

### Windows

Visual Studio の C++ ビルドツールと WebView2（Windows 11 なら同梱）。

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools
winget install --id Rustlang.Rustup
winget install --id OpenJS.NodeJS.LTS
```

### Linux (Debian / Ubuntu)

CI と同じものを入れる。

```bash
sudo apt-get update && sudo apt-get install -y --no-install-recommends \
  build-essential curl wget file pkg-config \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev libssl-dev libxdo-dev \
  libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

クリップボード操作の道具も入れておく（テストデータの投入に使う）。

```bash
sudo apt-get install -y xclip wl-clipboard
```

## 動かす

開発ビルドで確認するのがいちばん早い。ログが stderr に出る（リリースビルドは既定で無出力）。

```bash
npm run tauri dev
```

配布物そのものを確認したいときは、署名鍵なしでビルドできる。

```bash
npm run tauri build -- --bundles nsis
```

> `tauri.conf.json` の `createUpdaterArtifacts` が有効なので、`TAURI_SIGNING_PRIVATE_KEY` が
> 無いと**バンドルは出来たあと署名段階で失敗する**。`.exe` や `.AppImage` 自体は生成されて
> いるので、起動確認には使える。

## テストデータの投入

見えない文字を仕込んだ文字列を流し込む。全角空白・タブ・CRLF・LF・NBSP・ゼロ幅スペースが
一度に入る。

**Windows (PowerShell)**

```powershell
$t = "カタカナ`u{3000}全角スペース`tTAB`r`nCRLF行`n LF行`u{00a0}NBSP`u{200b}ZWSP"
Set-Clipboard -Value $t
```

**Linux (X11)**

```bash
printf 'カタカナ　全角スペース\tTAB\r\nCRLF行\n LF行\xc2\xa0NBSP\xe2\x80\x8bZWSP' | xclip -selection clipboard
```

**Linux (Wayland)**

```bash
printf 'カタカナ　全角スペース\tTAB\r\nCRLF行\n LF行\xc2\xa0NBSP\xe2\x80\x8bZWSP' | wl-copy
```

書式付き（HTML）の項目はブラウザで太字や色の付いた範囲をコピーするのが確実。「書式あり」の
警告アイコンが付く。

### 秘匿マークの確認（要件 1.8）

macOS には `scripts/macos-concealed-copy.swift` があるが、他 OS 用のヘルパーは無い。
clipbuf が見ているマーカーは `src-tauri/src/clipboard/conceal.rs` の 3 つで、OS ごとに
こう対応する。

| OS | マーカー | 確認方法 |
|---|---|---|
| Windows | `ExcludeClipboardContentFromMonitorProcessing` | 実際のパスワードマネージャでコピーする |
| KDE | `x-kde-passwordManagerHint` | 同上、または KDE の Plasma Vault / KWallet |
| macOS | `org.nspasteboard.ConcealedType` | 上記スクリプト |

Windows / KDE 用のヘルパーが欲しければ書けるので言ってほしい。

## 言語の確認

日本語でも英語でもないロケールでは英語になる（要件 12.3）。トレイメニューとウィンドウ
タイトルも翻訳対象（12.5）なので、そこも見る。

```bash
LANG=de_DE.UTF-8 npm run tauri dev   # UI もトレイも英語
LANG=ja_JP.UTF-8 npm run tauri dev   # 両方日本語
```

## 見つけたことの扱い

[platform-checklist.md](platform-checklist.md) の該当項目に沿って「操作 → 期待」で記録し、
期待と違ったら要件番号を添えて
[Issues](https://github.com/YasuhiroInagawa/clipbuf/issues) に出す。要件番号は
`.kiro/specs/clipbuf-mvp/requirements.md` にある。

実装中に踏んだ落とし穴（updater の署名バージョン照合、未登録シークレットが空文字に展開される
件、AppImage と glibc など）は [implementation_workflow.md](implementation_workflow.md) に
まとめてあるので、同じところで詰まったらまずそこを見る。
