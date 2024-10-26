---
layout: default
title: データフロー
---

# フェデレーションモデルの詳細設計

## データフロー

![データフローチャート](./data_flow.png)

## 各ステップの詳細説明

### 1. 送信用index.htmlからトランザクションデータ受信
ユーザーが送信用フォームからトランザクションデータを送信します。このデータには市町村（municipality）と大陸（continent）の情報が含まれます。

### 2. app.pyでデータ処理
送信されたトランザクションデータをapp.pyが受信し、`municipality`と`continent`を抜き出します。

### 3. 大陸の選択
抜き出した`continent`情報に基づいて、該当する大陸のチェーン（municipal_chain）を選択します。

### 4. municipal_chainでの処理
選択された大陸の市町村チェーンでトランザクションを処理し、`send_pending`に保存します。

### 5. ブロック生成と承認
`send_pending`に保存されたトランザクションが一定数（例：500件）に達すると、ブロックが生成され、承認されます。

### 6. continental_main_chainへの転送
承認されたブロックはcontinental_main_chainに転送され、保留リストに保存されます。

### 7. 受信側のプロセス
ユーザーが受信用フォームから信号を送信し、app.pyが信号を処理します。continental_main_chain経由で保留リストにアクセスし、10大陸に共有されます。

### 8. municipal_chainでの受信トランザクション処理
該当するmunicipal_chainで受信トランザクションを処理し、`receive_pending`に保存します。ブロックが生成・承認されると`complete`に更新され、MongoDBに保存されます。

### 9. 後続処理
保存されたデータは分析用データベースへの移行、Gossipアルゴリズムによる他の市町村とのデータ共有、他のDAppsからのデータ抽出などに利用されます。

```mermaid
graph TD
    %% 送信側のプロセス
    A[送信用index.htmlからトランザクションデータ受信] --> B[app.pyでデータ受信]
    B --> C[municipalityとcontinentの抽出]
    C --> D{大陸の選択}
    D -->|Asia| E[Asia_municipal_chainへ送信]
    D -->|Europe| F[Europe_municipal_chainへ送信]
    D -->|Default| G[Default_municipal_chainへ送信]
    E --> H[send_pendingにトランザクション保存]
    F --> H
    G --> H
    H --> I{ブロック生成条件確認}
    I -->|条件達成| J[ブロック生成]
    J --> K[ブロック承認]
    K --> L[continental_main_chainにブロック転送]
    L --> M[保留リストにブロック保存]

    %% 受信側のプロセス
    M --> N[受信用index.htmlから受信信号受信]
    N --> O[app.pyで受信信号処理]
    O --> P[continental_main_chain経由で保留リストにアクセス]
    P --> Q[10大陸に保留リスト共有]
    Q --> R{該当municipal_chainで受信トランザクション処理}
    R -->|処理成功| S[receive_pendingにトランザクション保存]
    S --> T{ブロック生成条件確認}
    T -->|条件達成| U[ブロック生成]
    U --> V[ブロック承認]
    V --> W[completeに更新]
    W --> X[MongoDBにトランザクション保存]
    X --> Y[後続処理へデータ移行]

    %% 後続処理
    Y --> Z[分析用データベースへの移行]
    Y --> AA[Gossipアルゴリズムによるデータ共有]
    Y --> AB[他のDAppsからのデータ抽出]

### 10. 小松市のAさんから金沢市のBさんに200愛貨を送信する一連の設定
DApps側とMunicipal Chain側の設定を踏まえて、小松市のAさんが金沢市のBさんに200愛貨を送信するための一連のセットアップ手順を再構成します。今回のケースでは、DAppsのFlaskとMongoDB、Municipal ChainのFlaskとMongoDB、さらにAsiaのMongoDBの全てが必要です。以下に、具体的な手順を一からご案内します。

1. 必要なMongoDBインスタンスを起動する
DApps側（デフォルトまたはAsia）のMongoDBインスタンスと、Municipal Chain側のAsia MongoDBインスタンスをそれぞれ起動します。
DApps MongoDB
ポート番号は、設定ファイルに基づいて起動します。ここではデフォルトのMongoDB（ポート27017）を利用しますが、必要に応じてAsiaのMongoDB（10024）も使用可能です。
D:\MongoDB\Server\7.0\bin\mongod.exe --port 27017 --dbpath "D:\data\mongodb\dapps"

Municipal Chain MongoDB (Asia)
D:\MongoDB\Server\7.0\bin\mongod.exe --port 10024 --dbpath "D:\data\mongodb\asia"

2. Flaskアプリのセットアップと起動
DApps Flask
DAppsのFlaskアプリケーションを起動します。環境変数または設定ファイルからポートを決定し、DAppsのFlaskアプリがリクエストを受け取れるようにします。
・Flask CLI（コマンドラインインターフェース）での実行: こちらの方法は、まず環境変数としてFLASK_APPにファイル名を設定し、その後Flask CLIを使ってアプリを実行します。
・追加機能の利用: Flask CLIを使用することで、Flaskが提供する追加機能（デバッグモードの切り替え、サーバーのポート指定、その他の設定）が簡単に利用できます。柔軟にアプリケーションを起動したい場合には、python app.pyを使います。CLIの機能を活用して、複雑な操作やデバッグを行いたい場合には、set FLASK_APP=app.py && flask run --port=1024が便利です。
set FLASK_APP=app.py
flask run --port=1024
（1024はAsia大陸のポート）

Municipal Chain Flask (Komatsu)
set FLASK_APP=app.py
flask run --port=2001

Municipal Chain Flask (Kanazawa)
set FLASK_APP=app.py
flask run --port=2000

3. ファイアウォール設定の確認（必要に応じて）
必要なポート（27017, 10024, 20000,20001, 2001, 2000など）を開放し、通信を許可します。
New-NetFirewallRule -DisplayName "Open MongoDB Port 10024" -Direction Inbound -LocalPort 10024 -Protocol TCP -Action Allow
New-NetFirewallRule -DisplayName "Open MongoDB Port 20000" -Direction Inbound -LocalPort 20000 -Protocol TCP -Action Allow
New-NetFirewallRule -DisplayName "Open MongoDB Port 20001" -Direction Inbound -LocalPort 20001 -Protocol TCP -Action Allow
New-NetFirewallRule -DisplayName "Open Flask Port 2001" -Direction Inbound -LocalPort 2001 -Protocol TCP -Action Allow
New-NetFirewallRule -DisplayName "Open Flask Port 2000" -Direction Inbound -LocalPort 2000 -Protocol TCP -Action Allow

4. DApps Flask アプリのトランザクション送信
app.pyから、送信リクエストを行います。具体的には、KomatsuのFlaskにアクセスしてトランザクションを処理し、Municipal Chain (KomatsuとKanazawa)とAsia MongoDBにデータが保存されます。
# DAppsでの送信リクエスト (例)
import requests

transaction_data = {
    "sender": "A",
    "sender_municipality": "Asia-Komatsu",
    "receiver": "B",
    "receiver_municipality": "Asia-Kanazawa",
    "amount": 200,
    "continent": "Asia",
    # その他必要なフィールドを追加
}

response = requests.post("http://localhost:2001/send", json=transaction_data)
print(response.json())

5. Municipal Chain側でのトランザクション受信と保存
Komatsu（小松市）のFlaskアプリがデータを受信し、Municipal Chain (Komatsu)側のMongoDBに送信情報が保存されます。その後、AsiaのMongoDBにデータが送信され、全てのチェーンでトランザクションが保持されます。

6. 全体のデータフロー確認とテスト
すべてのFlaskとMongoDBが起動し、必要な通信が可能であることを確認します。
トランザクションデータがMunicipal ChainのFlaskに保存されていることを確認します。
以上で、小松市から金沢市への200愛貨の送信が可能になるはずです。

### 11. 小松市のAさんから金沢市のBさんに200愛貨を送信するmunicipal_chainの実行プログラム
cargo run -- Asia-Komatsu というコマンドは、municipal_chainのRustプログラムを起動して、**Asia-Komatsu**という引数を渡しています。これにより、アジア地域のKomatsu市の設定で、Municipal Chainのプログラムが動作するように設定されています。このプログラムは、以下のような役割を果たします：

Komatsu市の代表者選出とトランザクションの承認:

Asia-Komatsuに特定された代表者リストから、送信者と受信者の代表者が選出され、トランザクションの承認が行われます。
MongoDBとの接続:

municipal_chainのコード内で、Komatsu市用のMongoDBが起動され、愛貨トランザクションの記録や、ペンディング状態のトランザクションを管理します。
送信トランザクションは、Komatsu市のMongoDBに保存され、その後Asiaのコンチネンタルチェーンに渡されます。
Flaskとの連携:

Komatsu市に関連するFlaskのポート（ここでは1024）が開かれており、トランザクションのリクエストを受け付けて処理するために必要です。
コマンド実行時のエラーメッセージと警告
警告メッセージ: cargo runの実行中に多数の警告が表示されていますが、これらは使われていないインポートや未使用の変数に関するもので、直接的な動作には影響しません。
エラー (STATUS_ACCESS_VIOLATION):
これはプログラムのスタックオーバーフローが原因で発生しています。解決策として、プログラムのスレッドスタックサイズを増加させるか、再帰処理やループ構造に無限ループがないか確認する必要があります。
再設定と起動手順の流れ
Rustコードの修正: municipal_chainのコードを再確認し、未使用の変数やデッドコードを削除し、無限ループがないか確認します。
MongoDBとFlaskの起動:
Flaskのポート (1024)とKomatsu市のMongoDB (10024) を開放し、他の必要なDAppsやMunicipal Chainのサービスを立ち上げます。
Komatsu市用に再起動: 上記の修正を行った後に、再度 cargo run -- Asia-Komatsu を実行して、Komatsu市のプログラムが正しく稼働することを確認します。
これで、Komatsu市（小松市）のAさんが送信する愛貨が、Asia地域のMongoDBとFlaskを介してトランザクションとして管理されるようになるはずです。

##　immudbへの書き込み方法
1. immudbの利用について
これでimmuclientを使用してimmudbに接続し、データベース操作を行うことができます。

2. ブロックチェーンからimmudbにデータを書き込む方法
2.1 immudbのアドレス設定
ブロックチェーンのアプリケーションからimmudbにデータを書き込むためには、immudbサーバーのアドレスとポートを正しく設定する必要があります。

デフォルトのアドレスとポート：

アドレス：localhostまたは127.0.0.1
ポート：3322
2.2 アプリケーションからの接続方法
言語ごとのクライアントライブラリ：
immudbは、さまざまなプログラミング言語向けのクライアントライブラリを提供しています。これらを使用して、アプリケーションからimmudbに接続し、データ操作を行うことができます。
Go
Java
Python
Node.js
Rust（もし公式ライブラリがない場合、gRPCを使用して接続可能）

2.3 接続の設定例（Pythonの場合）
ステップ1：Pythonクライアントライブラリのインストール
pip install immudb-py
ステップ2：Pythonコードでの接続
from immudb.client import ImmudbClient

# immudbサーバーのアドレスとポートを指定
client = ImmudbClient("{immudbのアドレス}", port=3322)

# ログイン
client.login("immudb", "immudb")

# データのセット
client.set("mykey", "myvalue")

# データの取得
value, _ = client.get("mykey")
print(value)  # 出力: myvalue
注意点：
{immudbのアドレス}には、immudbサーバーの実際のIPアドレスまたはホスト名を指定します。
Dockerでimmudbを実行している場合、ホストマシンから接続する場合はlocalhostを使用できます。

2.4 Rustからの接続
immudbの公式Rustクライアントライブラリは提供されていない可能性がありますが、gRPCを使用して接続できます。
ステップ1：gRPCクライアントのセットアップ
RustでgRPCを使用するために、tonicやgrpcioなどのライブラリを使用します。

ステップ2：immudbのプロトコル定義を使用
immudbのプロトコルバッファ定義（.protoファイル）を取得し、Rustコード内で使用します。

3. Webインターフェースでデータを確認する方法
immudbには、Webベースの管理コンソールが提供されています。これを使用して、データベースの内容を確認したり、操作を行ったりできます。

3.1 管理コンソールへのアクセス
URL：http://localhost:9497
Dockerでimmudbを起動する際に、ポート9497を公開している必要があります。
⇒　docker run -d --name immudb -p 3322:3322 -p 9497:9497 codenotary/immudb
3.2 ログイン
ユーザー名：immudb
パスワード：immudb
3.3 管理コンソールの機能
データの閲覧：データベース内のキーと値を確認できます。
クエリの実行：GUI上でSQLクエリを実行できます。
統計情報：サーバーの状態や統計情報を確認できます。
3.4 注意事項
テスト用途：

管理コンソールはテストやデバッグに便利ですが、本番環境ではセキュリティ上の理由から適切な認証とアクセス制御を行ってください。
4. ブロックチェーンアプリケーションとの統合
ブロックチェーンからimmudbにデータを書き込むシナリオでは、以下の点を考慮する必要があります。

4.1 データフローの設計
ブロックチェーンイベントのキャプチャ：

スマートコントラクトからのイベントやトランザクションデータをキャプチャします。
ミドルウェアの開発：

ブロックチェーンとimmudbの間にミドルウェアを設け、データを変換・転送します。
4.2 接続設定
アドレスとポートの設定：

ミドルウェアやアプリケーションからimmudbに接続する際、immudbのアドレス（IPアドレスまたはホスト名）とポート3322を指定します。
Docker環境での接続：

同一ホスト内での接続：

アプリケーションがホストマシン上で実行されている場合、localhostを使用してimmudbに接続できます。
Dockerコンテナ間での接続：

アプリケーションもDockerコンテナで実行している場合、同じDockerネットワーク上に配置し、コンテナ名で通信できます。
4.3 セキュリティの考慮
認証情報の管理：

immudbのユーザー名とパスワードを安全に管理します。
TLS/SSLの使用：

通信を暗号化するために、immudbのTLS設定を有効にすることを検討してください。
5. テスト環境での確認
テスト目的でimmudbの動作を確認するには、以下の方法があります。

5.1 管理コンソールの利用
前述の通り、Webブラウザからデータを確認できます。
5.2 CLIツールの使用
immuclientを使用して、コマンドラインからデータ操作や確認ができます。

# データのセット
immuclient set mykey myvalue

# データの取得
immuclient get mykey
5.3 アプリケーションからのログ出力
アプリケーションコード内で、immudbへの書き込みや読み込み結果をログ出力することで、動作確認ができます。

# MongoDBの状態遷移について
トランザクションの状態管理（send, send_pending, receive, receive_pending, complete）は理にかなっており、特にトランザクションの各段階を明確に追跡できるという点で良いアイデアです。

具体的には、以下のようにトランザクションの状態を管理することが考えられます：
1. 送信フロー
状態: send
送信者がトランザクションを生成し、dapps\app.pyからMunicipal Chainに送信するときに、まずMongoDBにsend状態で保存される。
ここでは、トランザクションが送信されたが、Municipal Chainによってまだ承認されていないことを示す。

状態: send_pending
Municipal Chainで承認され、トランザクションがsend_pending状態に移行される。
この状態は、受信者側が受け取る準備が整うまで、つまり受信側からの確認を待っていることを示す。

2. 受信フロー
状態: receive
受信者が受信リクエストを送信すると、dapps\app.pyからMunicipal Chainに送信され、MongoDBにはreceive状態で保存される。
ここでは、受信リクエストが送信されたが、まだMunicipal Chainで承認されていないことを示す。

状態: receive_pending
Municipal Chainで受信が承認されると、receive_pending状態になる。
この状態は、トランザクションがブロックチェーンに取り込まれる準備が整ったことを示す。

3. 最終状態
状態: complete
トランザクションがブロックチェーンに記録され、承認されれば、complete状態に変更される。
これにより、そのトランザクションは完全に処理済みであることが確認できる。

メリット:
各トランザクションの状態を段階的に追跡でき、問題が発生した場合（例えば、承認エラーや処理中断）に適切な対応が取れる。
取引のステータス管理がしやすくなり、どの段階で問題が起きているかを簡単に確認できる。
MongoDBのトランザクションステータスを確認するだけで、トランザクションがどの段階にあるのかを簡単に把握できる。


# GitHubにコードをプッシュして共有する方法を一つずつ説明します。
1. GitHubアカウントの作成
まずは、GitHubアカウントが必要です。まだ持っていない場合は、GitHubでアカウントを作成してください。

2. 新しいリポジトリの作成
GitHubにログインした後、右上の「+」アイコンをクリックし、「New repository」を選択します。
リポジトリ名を入力します（例：city_chain_project）。
「Private」または「Public」を選択します。**「Public」を選ぶと誰でもアクセス可能になり、「Private」**だと限られたユーザーだけがアクセスできます。「Create repository」をクリックします。

3. Gitの初期設定
ローカルの開発環境で次の設定を行いましょう。まず、ターミナルやコマンドプロンプトを開き、以下のコマンドを実行して、Gitのユーザー名とメールアドレスを設定します。
git config --global user.name "Your GitHub Username"
git config --global user.email "your-email@example.com"

4. ローカルリポジトリの初期化
city_chain_projectフォルダに移動します。
cd path/to/your/city_chain_project

Gitの初期化を行います。
git init

5. GitHubリポジトリをローカルリポジトリにリンク
GitHubのリポジトリページで表示される「https://github.com/username/repositoryname.git」のようなURLをコピーします。
ターミナルで次のコマンドを実行してリポジトリを追加します。
git remote add origin https://github.com/username/city_chain_project.git

6. ファイルを追加・コミットする
すべてのファイルをステージングします。
git add .

コミットメッセージを追加してコミットします。
git commit -m "Initial commit"

7. GitHubにプッシュする
次に、ローカルの変更をGitHubにプッシュします。
git push -u origin main

これで、GitHubリポジトリにコードがアップロードされ、共有が完了します。