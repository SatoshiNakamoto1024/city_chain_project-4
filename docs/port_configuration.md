### 1. ポートの説明
flask_port: これは大陸レベルでのFlaskサーバーのポートです。つまり、大陸全体を管理するメインチェーン（Continental Main Chain）でFlaskサーバーがリッスンするポートです。

city_port: これは市町村チェーン（Municipal Chain）での通信に使われるポートです。このポートは、市町村のブロックチェーンノードがリッスンするために使用され、市町村間のブロックチェーン通信に使われます。

city_flask_port: これは、市町村レベルで動作するFlaskサーバーのポートです。DAppsや他のサービスが市町村チェーンと通信するときに使用されます。

したがって、flask_portは大陸レベル、city_portは市町村のブロックチェーン通信、city_flask_portは市町村レベルのFlaskサーバー用です。

### 2. 全体の流れとポートの役割
 index.htmlからapp.pyへデータ送信
index.htmlのフォームから送信ボタンを押すと、JavaScriptによってトランザクションデータがapp.pyに送信されます。
使用ポート: app.pyのFlaskサーバーがリッスンしているポート（flask_port）。たとえば、大陸が「Asia」の場合、municipalities.jsonに設定された 1024 が使われます。

 app.pyからmunicipal_chainへデータ送信
app.pyはデータを受信後、送信者と受信者の市町村データに基づいて、municipal_chainの適切なURLを構築し、データをPOSTリクエストとして送信します。
このとき、municipal_chainが待ち受けているポート（city_port）が使われます。たとえば、Taipeiのcity_portは 20000 です。

 municipal_chainでデータ受信
municipal_chain側のRustプログラムがcity_port（例: 20000）でトランザクションデータを受け取り、ブロックチェーンへの記録やDPoS承認などの処理を行います。

 具体的な流れ
index.html → (送信) → app.py: flask_port（例: 1024）
app.py → (POSTリクエスト) → municipal_chain: city_port（例: 20000）

 ポイント
flask_port は、ユーザーが index.html を通して app.py にアクセスするときに使用されるポートです。
city_port は、app.py から municipal_chain にデータを送るときに使われるポートです。
このように、index.htmlからのフロントエンド通信には flask_port が、バックエンドの municipal_chain 通信には city_port が使われています。

### 3. ポートの開き方
管理者としてPowershellに入り、以下のように記載することで、windowsのファイアウォール：Port 20000 を開くことができる。
これは、他のサーバーやクライアントがポート 20000 を通じて接続する必要がある場合です。通常、APIサーバー（Flaskサーバーなど）やデータベースサーバーが外部と通信するために特定のポートを開放することが求められます。
PS C:\WINDOWS\system32> New-NetFirewallRule -DisplayName "Open Port 20000" -Direction Inbound -LocalPort 20000 -Protocol TCP -Action Allow

Flaskポートの開き方は下記のとおり（Kanazawaの場合）
上記のNew-NetFirewallRule は外部からのアクセスを許可します。下記のflask run --port=2000 は、Flaskアプリケーションが指定したポートで動作を開始します。両者は別の役割を果たしているので、外部からの接続が必要な場合には 両方の設定が必要です。つまり、Flaskアプリを指定ポートで起動し、さらにそのポートへの外部アクセスを許可するためにファイアウォールでポートを開ける必要があります。
export FLASK_APP=app.py
flask run --port=2000

MongoDBを起動するのは下記のとおり（Asiaの場合）
D:\MongoDB\Server\7.0\bin\mongod.exe --config "D:\city_chain_project\config\mongodb_Asia.conf"

MongoDBの稼働チェックは下記のとおり（10024の場合）
mongo --port 10024 --eval "db.stats()"

immudbの場合は、下記のようにポートを開く。
⇒Dockerでimmudbを起動する際に、ポート9497を公開している必要があります。
docker run -d --name immudb -p 3322:3322 -p 9497:9497 codenotary/immudb

Dockerでimmudbを実行する際に、フォルダを指定して永続化するためには、-vオプションを使ってホストマシンのフォルダをコンテナ内のフォルダにマウントする必要があります。以下はその例です。
docker run -d --name immudb -p 3322:3322 -p 9497:9497 -v /path/to/local/folder:/var/lib/immudb codenotary/immudb

# 全体をまとめたテストスクリプト例
テスト時には以下のようなスクリプトを作成して、すべての大陸と市町村を対象に一括でテストを実行できるようにします。
・起動スクリプト例
docker-compose up -d --build

・テスト実行 (例)
python run_tests.py  # 各大陸チェーンと市町村チェーンへのトランザクション送信テストを実行

・終了
docker-compose down