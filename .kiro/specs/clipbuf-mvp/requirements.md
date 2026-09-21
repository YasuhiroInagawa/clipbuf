# Requirements Document

## Introduction

clipbuf は、デスクトップ（Windows / macOS / Linux）でアプリ間のコピー＆ペーストを行うユーザー向けのクリップボードバッファアプリである。他アプリでコピーされたテキストを自動で取り込み、直近 N 件を 1 行ずつリスト表示し、各項目に含まれる空白・改行・不可視文字を記号で可視化し、書式（スタイル）の有無・機種依存文字・制御文字などの「貼り付け事故」要因を警告アイコンで示す。ユーザーはウィンドウ上のトグルで無害化オプション（スタイル削除、改行処理、トリムなど）を選び、1 クリックで項目をクリップボードへ書き戻して貼り付けできる。

既存のクリップボード履歴ツールとの差別化は「可視化」「警告」「無害化転送」の 3 点であり、本仕様（clipbuf-mvp）はその初期リリースの全機能を対象とする。

## Boundary Context

- **In scope**:
  - 他アプリでのコピー（テキスト）の自動取り込みと、直近 N 件の一時保持
  - 1 行プレビューでの不可視文字の可視化と警告アイコン表示
  - 転送オプション付きのクリップボードへの書き戻し
  - 常に最前面のフローティングウィンドウ、グローバルホットキー、トレイ / メニューバー常駐
  - 保持件数・ホットキー・言語などの設定の永続化
  - 3 OS 向けの配布物の生成と、アプリ内からの更新通知
- **Out of scope**（初期リリースでは扱わない）:
  - 画像・ファイル・その他非テキスト形式の履歴
  - 履歴の永続化・検索・ピン留め・お気に入り
  - クラウド同期、ネットワーク経由の共有
  - 画面端へのドッキング（他ウィンドウが避ける領域の確保）
  - Flatpak / Mac App Store / Microsoft Store での配布
  - 取り込んだテキストの編集
- **Adjacent expectations**:
  - clipbuf は OS のクリップボードの変化を観測して取り込む。コピー操作そのものを横取り・改変することはなく、コピー元アプリと貼り付け先アプリの挙動には介入しない。
  - パスワードマネージャ等が付与する「監視対象外」の印は OS / アプリ側の慣習に従って尊重する。印を付けるのは clipbuf の責務ではない。
  - macOS では OS 側がクリップボード読み取りに対して許可ダイアログを出す場合がある。この許可の取得はユーザーの操作に委ね、clipbuf は案内のみ行う。
  - Linux の Wayland 環境では、コンポジタがフォーカス外のクリップボード読み取りを許可しない場合に取り込みができない。これは clipbuf 側で回避しない。

## Requirements

### Requirement 1: クリップボード監視と自動取り込み
**Objective:** As a ユーザー, I want 他アプリでコピーしたテキストが自動で clipbuf に取り込まれること, so that コピーのたびに clipbuf を操作しなくても履歴が溜まる

#### Acceptance Criteria
1. While clipbuf が起動している, when 他アプリでテキストがクリップボードにコピーされる, the clipbuf shall そのテキストを新しい項目としてリストの先頭に追加する
2. When クリップボードにテキストと同時に書式付きデータ（HTML / RTF 相当）が載っている, the clipbuf shall テキスト本文と書式付きデータの両方を項目に保持する
3. When クリップボードの内容がテキストを含まない（画像・ファイルのみなど）, the clipbuf shall その変化を無視し、項目を追加しない
4. When clipbuf 自身がクリップボードへ書き込んだ, the clipbuf shall その変化を取り込み対象から除外する
5. When クリップボードの内容にパスワードマネージャ等による「監視対象外」の印が付いている, the clipbuf shall その内容を取り込まない
6. When 直前に取り込んだ項目と同一内容のテキストが再びコピーされる, the clipbuf shall 重複する項目を追加しない
7. While clipbuf のウィンドウが非表示である, the clipbuf shall 取り込みを継続する
8. The clipbuf shall 他アプリでのコピーから 1 秒以内にリストへ反映する

### Requirement 2: リストの保持と管理
**Objective:** As a ユーザー, I want 直近の項目だけが常に残っていること, so that リストが際限なく増えず、必要な直近のものをすぐ選べる

#### Acceptance Criteria
1. The clipbuf shall 項目を新しい順に並べ、設定された保持件数 N を上限として保持する
2. When 項目数が保持件数 N を超える, the clipbuf shall 最も古い項目から削除する
3. When ユーザーが項目の削除操作を行う, the clipbuf shall その項目のみをリストから削除する
4. When ユーザーが全削除操作を行う, the clipbuf shall リストのすべての項目を削除する
5. When 保持件数 N の設定が現在の項目数より小さい値に変更される, the clipbuf shall 古い項目から削除して N 件に収める
6. The clipbuf shall 保持件数 N の既定値を 20 とする

### Requirement 3: 項目の 1 行表示と不可視文字の可視化
**Objective:** As a ユーザー, I want 各項目に含まれる空白・改行・不可視文字が見えること, so that 貼り付け前に事故要因に気づける

#### Acceptance Criteria
1. The clipbuf shall 各項目を 1 行で表示し、行の右側に警告アイコン列を並べる
2. When 項目のテキストが改行を含む, the clipbuf shall 表示上は改行を改行記号に置き換えて 1 行に連結し、データ本体は改行を含んだまま保持する
3. The clipbuf shall 項目のテキストを等幅フォントで表示する
4. The clipbuf shall 半角空白・全角空白・タブ・改行・NBSP・ゼロ幅スペース等の不可視文字を、それぞれ区別できる記号と色で表示する
5. The clipbuf shall 全角空白を半角空白と異なる記号または色で表示する
6. The clipbuf shall 行の幅に収まる範囲でテキストの先頭から可能な限り多くの文字を表示する

### Requirement 4: 項目テキストの閲覧（読み取り専用スクロール）
**Objective:** As a ユーザー, I want 1 行に収まらない長いテキストも横にスクロールして確認できること, so that 全文を見てから転送を判断できる

#### Acceptance Criteria
1. When 項目のテキストが行の幅に収まらない, the clipbuf shall その行を横スクロールして全文を閲覧できるようにする
2. The clipbuf shall 項目のテキスト表示を編集不可とする
3. The clipbuf shall 項目のテキスト表示上での文字列選択とコピーを許可しない
4. When 横スクロールした項目からフォーカスが外れる, the clipbuf shall その項目のスクロール位置を先頭に戻す

### Requirement 5: 警告ステータスの表示
**Objective:** As a ユーザー, I want 貼り付け事故につながる特徴が一目で分かること, so that 無害化オプションを選ぶ判断ができる

#### Acceptance Criteria
1. When 項目に書式付きデータが含まれる, the clipbuf shall 「スタイルあり」の警告アイコンを表示する
2. When 項目のテキストの先頭または末尾に空白文字または改行がある, the clipbuf shall 「先頭・末尾に空白/改行あり」の警告アイコンを表示する
3. When 項目のテキストにタブ文字が含まれる, the clipbuf shall 「タブあり」の警告アイコンを表示する
4. When 項目のテキストに機種依存文字（NEC / IBM 拡張文字など、JIS X 0208 の範囲外で環境により表示が変わる文字）が含まれる, the clipbuf shall 「機種依存文字あり」の警告アイコンを表示する
5. When 項目のテキストにタブ・改行以外の制御文字、または不正なサロゲートペアが含まれる, the clipbuf shall 「バイナリ/制御文字あり」の警告アイコンを表示する
6. When 項目のテキストに複数種類の改行コード（CRLF / LF / CR）が混在する, the clipbuf shall 「改行コード混在」の警告アイコンを表示する
7. When 項目のテキストに BOM、双方向制御文字、または NFC と NFD の混在が含まれる, the clipbuf shall 「文字表現の注意」の警告アイコンを表示する
8. When ユーザーが警告アイコンにポインタを合わせる, the clipbuf shall その警告の意味を示す説明を表示する
9. When 項目が警告に該当しない, the clipbuf shall その警告のアイコンを表示しない

### Requirement 6: クリップボードへの転送操作
**Objective:** As a ユーザー, I want リストの項目を 1 操作でクリップボードに戻せること, so that すぐに貼り付けできる

#### Acceptance Criteria
1. When ユーザーが項目の行をクリックする, the clipbuf shall その時点の転送オプションを適用した内容をクリップボードへ書き込む
2. When ユーザーがキーボードで項目を選択して Enter を押す, the clipbuf shall クリックと同じ転送を行う
3. The clipbuf shall キーボードの上下操作でリスト内の選択項目を移動できるようにする
4. When ユーザーが項目の行にポインタを合わせる, the clipbuf shall 「プレーンテキストで転送」と「元のまま転送」の代替アクションを表示する
5. When ユーザーが「プレーンテキストで転送」を選ぶ, the clipbuf shall 転送オプションの設定にかかわらず、書式を除いたテキスト本文のみを書き込む
6. When ユーザーが「元のまま転送」を選ぶ, the clipbuf shall 転送オプションの設定にかかわらず、取り込んだ内容（テキスト本文と書式付きデータ）をそのまま書き込む
7. When 転送が完了する, the clipbuf shall 転送した行を短時間ハイライトして完了を示す
8. When 転送が完了する, the clipbuf shall ウィンドウの表示状態とリストの順序を変更しない
9. The clipbuf shall 転送によってリスト上の元の項目のデータを変更しない
10. If クリップボードへの書き込みに失敗する, the clipbuf shall 失敗をユーザーに通知する

### Requirement 7: 転送オプション
**Objective:** As a ユーザー, I want 貼り付け事故の原因になる書式・空白・改行を転送時に取り除けること, so that 貼り付け先を汚さない

#### Acceptance Criteria
1. The clipbuf shall 以下の転送オプションをリストと同じウィンドウ上に常時表示のトグルとして提供する：スタイルの保持/削除、改行の処理（そのまま/削除/空白に置換）、先頭末尾の空白・改行のトリム、タブの空白への変換、全角空白の半角空白への変換
2. When 転送が行われる, the clipbuf shall その時点のトグル状態を適用する（項目ごとにオプションを持たない）
3. The clipbuf shall トグル状態をアプリ全体で共通の設定として保持し、アプリの再起動後も維持する
4. When 「スタイルの削除」が有効で転送される, the clipbuf shall 書式付きデータを含めず、テキスト本文のみを書き込む
5. When 「スタイルの保持」が有効で、かつ項目に書式付きデータが含まれ、かつ改行の処理・トリム・タブ変換・全角空白変換のいずれかが有効な状態で転送される, the clipbuf shall 書式付きデータとテキスト本文をそのまま書き込み、テキスト変換を適用しなかった旨をユーザーに通知する
6. When 「スタイルの保持」が有効で、かつ項目に書式付きデータが含まれない状態で転送される, the clipbuf shall 有効なテキスト変換をテキスト本文に適用して書き込む
7. When 「改行の削除」が有効で転送される, the clipbuf shall テキスト本文からすべての改行を取り除く
8. When 「改行の空白置換」が有効で転送される, the clipbuf shall テキスト本文の各改行（CRLF は 1 つとして扱う）を半角空白 1 つに置き換える
9. When 「トリム」が有効で転送される, the clipbuf shall テキスト本文の先頭と末尾の空白文字（半角・全角・NBSP・タブ）と改行を取り除く
10. When 「タブの空白への変換」が有効で転送される, the clipbuf shall テキスト本文の各タブを設定された個数の半角空白に置き換える
11. When 「全角空白の半角空白への変換」が有効で転送される, the clipbuf shall テキスト本文の各全角空白を半角空白 1 つに置き換える
12. When 複数のテキスト変換が有効で転送される, the clipbuf shall 改行の処理、タブ変換、全角空白変換、トリムの順に適用する

### Requirement 8: ウィンドウとホットキー
**Objective:** As a ユーザー, I want clipbuf が邪魔にならない小さな最前面ウィンドウとして常駐し、必要なときにすぐ呼び出せること, so that 作業の流れを止めずに使える

#### Acceptance Criteria
1. The clipbuf shall 他アプリのウィンドウより常に前面に表示される小さなフローティングウィンドウとしてリストを表示する
2. When ユーザーがグローバルホットキーを押す, the clipbuf shall ウィンドウの表示と非表示を切り替える
3. While ウィンドウが非表示である, when ユーザーがグローバルホットキーを押す, the clipbuf shall ウィンドウを表示して最新の項目を選択状態にする
4. The clipbuf shall システムトレイ（macOS ではメニューバー）にアイコンを常駐させ、そこからウィンドウの表示、設定の表示、終了ができるようにする
5. When ユーザーがウィンドウを閉じる操作を行う, the clipbuf shall アプリを終了せずウィンドウを非表示にする
6. The clipbuf shall ウィンドウの位置とサイズをアプリの再起動後も維持する
7. The clipbuf shall ユーザーがウィンドウの大きさを変更できるようにする

### Requirement 9: 設定
**Objective:** As a ユーザー, I want 保持件数やホットキーなどを自分の使い方に合わせられること, so that 環境ごとに快適に使える

#### Acceptance Criteria
1. The clipbuf shall 設定画面で以下を変更できるようにする：保持件数 N、グローバルホットキー、タブ変換時の空白個数、クリップボード確認間隔（対応 OS のみ）、OS ログイン時の自動起動、UI 言語
2. The clipbuf shall 設定をアプリの再起動後も維持する
3. When 設定が変更される, the clipbuf shall 再起動なしで新しい設定を反映する
4. If ユーザーが指定したグローバルホットキーが他のアプリまたは OS に既に使用されている, the clipbuf shall 登録できなかった旨を表示し、直前の設定を維持する
5. When 「OS ログイン時の自動起動」が有効にされる, the clipbuf shall 次回の OS ログイン時にウィンドウを非表示のまま起動する
6. The clipbuf shall 転送オプションのトグルを設定画面には置かず、リストのウィンドウ上でのみ変更できるようにする

### Requirement 10: 履歴の非永続化とプライバシー
**Objective:** As a ユーザー, I want コピーした内容が端末に残ったり外部に送られたりしないこと, so that 機密情報を扱う作業でも安心して使える

#### Acceptance Criteria
1. The clipbuf shall 取り込んだ項目をメモリ上にのみ保持し、ディスクへ保存しない
2. When clipbuf が終了する, the clipbuf shall 取り込んだすべての項目を破棄する
3. The clipbuf shall 取り込んだ項目の内容をネットワークへ送信しない
4. The clipbuf shall 更新確認以外の目的でネットワーク通信を行わない
5. The clipbuf shall 取り込んだ項目の内容をログファイルへ出力しない

### Requirement 11: 対応プラットフォーム
**Objective:** As a ユーザー, I want Windows / macOS / Linux のどれでも同じ使い勝手で使えること, so that 複数の環境で同じ習慣を持てる

#### Acceptance Criteria
1. The clipbuf shall Windows、macOS、Linux の各デスクトップ環境で動作する
2. The clipbuf shall 3 OS で同一の機能とリスト表示を提供する
3. While Linux の X11 環境または XWayland 環境で動作している, the clipbuf shall 他アプリでのコピーを取り込む
4. While Linux の Wayland 環境でコンポジタがフォーカス外のクリップボード読み取りを許可している, the clipbuf shall 他アプリでのコピーを取り込む
5. If Linux の Wayland 環境でコンポジタがフォーカス外のクリップボード読み取りを許可しない, the clipbuf shall 取り込みができない旨をユーザーに通知する
6. If macOS でクリップボードの読み取りが OS により拒否される, the clipbuf shall 許可の設定方法をユーザーに案内する

### Requirement 12: UI 言語
**Objective:** As a ユーザー, I want UI が自分の言語で表示されること, so that 警告や通知の意味を正しく理解できる

#### Acceptance Criteria
1. The clipbuf shall 日本語と英語の UI を提供する
2. When 初回起動時に OS の言語が日本語である, the clipbuf shall 日本語 UI で起動する
3. When 初回起動時に OS の言語が日本語以外である, the clipbuf shall 英語 UI で起動する
4. When ユーザーが設定で UI 言語を選択する, the clipbuf shall OS の言語にかかわらず選択された言語で表示する

### Requirement 13: 配布と更新
**Objective:** As a ユーザー（および配布者）, I want 各 OS 向けの配布物を GitHub から取得して安全に導入・更新できること, so that 無料ツールとして継続的に使える

#### Acceptance Criteria
1. The clipbuf shall Windows 向けインストーラ、macOS 向けアプリ、Linux 向けパッケージ（AppImage / .deb / .rpm）を GitHub Releases で配布する
2. The clipbuf shall リリース用の配布物を継続的インテグレーションで自動生成し、手元でビルドした配布物を使わない
3. The clipbuf shall macOS 向け配布物に開発者署名と公証を施し、ユーザーが警告なしに起動できるようにする
4. The clipbuf shall Windows 向け配布物にコード署名を施さず、起動時の警告の回避手順を README に記載する
5. The clipbuf shall 各配布物の SHA-256 ハッシュをリリースに併記する
6. When clipbuf が起動する, the clipbuf shall 新しいバージョンの有無を確認し、あればユーザーに通知する
7. When ユーザーが更新を承諾する, the clipbuf shall 更新をダウンロードして適用する
8. If ユーザーが更新を承諾しない, the clipbuf shall 更新を適用せず現在のバージョンで動作を継続する
9. The clipbuf shall 依存ライブラリのライセンス一覧を配布物に同梱する
10. The clipbuf shall macOS のクリップボードプライバシー警告への対応方法を README に記載する
