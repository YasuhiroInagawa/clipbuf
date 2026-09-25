# clipbuf

貼り付ける前に、**見える・止められる・直せる**クリップボードバッファ。

他アプリでコピーしたテキストを自動で取り込み、直近 N 件を 1 行ずつ表示します。半角/全角空白、タブ、
改行、NBSP、ゼロ幅スペースといった見えない文字を記号と色で示し、書式や機種依存文字の有無を警告
アイコンで伝えます。行をクリックすると、選んだ無害化を適用したうえでクリップボードへ書き戻します。
元のデータは変更しません。

Windows / macOS / Linux。無料・オープンソース（MIT）。

## できること

- **自動取り込み** — 他アプリでのコピーを観測して FIFO で保持（既定 20 件）
- **可視化** — 空白・タブ・改行・NBSP・ゼロ幅文字などを記号と色で区別。CRLF / LF / CR も色分け
- **警告** — 書式あり、先頭末尾の空白、タブ、機種依存文字、制御文字、改行コード混在
- **無害化転送** — 書式削除、改行の削除/空白置換、トリム、タブ→空白、全角空白→半角をトグルで選択
- **常駐** — 最前面の小さなウィンドウ。グローバルホットキー（既定 `Alt+Shift+V`）とトレイアイコン

## 持ち出さない

- 取り込んだ内容は**メモリ上にのみ**保持します。ディスクに保存せず、終了時に破棄します
- 内容をネットワークへ送信しません。通信は**更新確認だけ**です
- 内容をログに出力しません
- パスワードマネージャ等が付ける「監視対象外」マークの付いた内容は取り込みません

## 導入

[Releases](https://github.com/YasuhiroInagawa/clipbuf/releases) から各 OS の配布物を取得してください。
各リリースには全ファイルの SHA-256 一覧（`SHA256SUMS.txt`）と、依存ライブラリのライセンス一覧
（`THIRD-PARTY.md`）を添付しています。

### Windows

`clipbuf_x.y.z_x64-setup.exe`（NSIS インストーラ）を実行します。

**「WindowsによってPCが保護されました」と表示されます。** clipbuf にはコード署名を行っていないため
です。署名証明書は個人が継続的に負担するには高額で、無料ツールとして配布する方針と釣り合わないと
判断しました。続行する場合は次の手順です。

1. 青いダイアログの「詳細情報」をクリック
2. 現れた「実行」ボタンをクリック

不安な場合は、ダウンロードしたファイルの SHA-256 が `SHA256SUMS.txt` の値と一致することを確認して
ください。

```powershell
Get-FileHash .\clipbuf_x.y.z_x64-setup.exe -Algorithm SHA256
```

### macOS

`clipbuf_x.y.z_universal.dmg` を開き、アプリケーションフォルダへドラッグします。Apple Silicon と
Intel の両方で動く universal バイナリです。開発者署名と公証を行っているため、警告なく起動できます。

**初回起動時に、他アプリのコピーを取り込めないことがあります。** macOS がクリップボードの読み取りを
制限している場合、clipbuf は「クリップボードへのアクセスが拒否されています」と表示します。次の手順で
許可してください。

1. システム設定 › プライバシーとセキュリティ › **ペーストボード**
2. 一覧の clipbuf をオンにする

clipbuf はメニューバーに常駐し、Dock には表示されません。

### Linux

AppImage / deb / rpm を用意しています。

```bash
chmod +x clipbuf_x.y.z_amd64.AppImage && ./clipbuf_x.y.z_amd64.AppImage
```

```bash
sudo dpkg -i clipbuf_x.y.z_amd64.deb
```

**Wayland では取り込みに制約があります。** clipbuf は他アプリのクリップボードを読むため、コンポジタが
フォーカス外のアプリによる読み取りを許可している必要があります。

| 環境 | 取り込み |
|---|---|
| X11 | 全アプリから取り込めます |
| KDE Plasma (Wayland) | `wlr-data-control` に対応しているため取り込めます |
| Sway / wlroots 系 (Wayland) | 同上 |
| GNOME (Wayland) | **取り込めません。** `wlr-data-control` を実装していないため、XWayland アプリからのコピーのみ取り込みます |

GNOME Wayland では、clipbuf が「Wayland で data-control が使えないため、X11/XWayland アプリからの
コピーのみ取り込みます」とウィンドウ上部に表示します。全アプリから取り込みたい場合は、ログイン画面で
X11 セッションを選択してください。

## 更新

起動時に新しいバージョンの有無を確認し、あればウィンドウ上部に通知します。「更新する」で適用、
「あとで」で現在のバージョンのまま使い続けます。更新の配布物は署名されており、署名が検証できない
場合は適用しません。

## 設定

トレイアイコン（macOS はメニューバー）から開きます。保持件数、グローバルホットキー、タブ幅、
ポーリング間隔（macOS / Wayland のみ）、ログイン時に起動、プレビューの折り返し、UI 言語（日本語 /
英語 / システムに従う）。

## 連絡先

質問・不具合報告は [Issues](https://github.com/YasuhiroInagawa/clipbuf/issues) へお願いします。

## 開発

```bash
npm install && npm run tauri dev
```

```bash
npm test && npm run check && npm run lint
```

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

依存ライブラリのライセンス一覧を更新するには `npm run licenses` を実行します。手動確認の項目は
[doc/platform-checklist.md](doc/platform-checklist.md) にあります。

## ライセンス

MIT。[LICENSE](LICENSE) を参照してください。依存ライブラリのライセンスは
[THIRD-PARTY.md](THIRD-PARTY.md) にまとめています。
