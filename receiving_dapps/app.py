from flask import Flask, request, jsonify, send_from_directory
from flask_cors import CORS
import requests
import os
import json
from pymongo import MongoClient
from datetime import datetime, timezone
from bson import json_util

app = Flask(__name__, static_folder='static')
CORS(app)  # クロスオリジンリソースシェアリングを許可
app.secret_key = 'your_secret_key'  # Flaskのセッション管理用
JWT_SECRET = 'jwt_secret_key'  # JWTのシークレット

users = {'user1': 'password123'}  # 簡易的なユーザー認証

# MongoDB設定ファイルパス
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
MONGODB_CONFIG_PATH = os.path.join(BASE_DIR, 'mongodb_config.json')

# MongoDB設定ファイルを読み込む
def load_mongodb_config():
    try:
        with open(MONGODB_CONFIG_PATH, 'r', encoding='utf-8') as file:
            return json.load(file)
    except FileNotFoundError:
        raise FileNotFoundError(f"Error: '{MONGODB_CONFIG_PATH}' not found. Please check the path.")
    except json.JSONDecodeError as e:
        raise ValueError(f"Error decoding JSON from '{MONGODB_CONFIG_PATH}': {e}")

mongodb_config = load_mongodb_config()

# MongoDB URI を取得する関数
def get_mongo_uri(instance_type, continent):
    if instance_type in mongodb_config and continent in mongodb_config[instance_type]:
        return mongodb_config[instance_type][continent]
    elif instance_type in mongodb_config and 'Default' in mongodb_config[instance_type]:
        return mongodb_config[instance_type]['Default']
    else:
        raise ValueError(f"MongoDB URI not found for instance type '{instance_type}' and continent '{continent}'")

def get_mongo_connection(instance_type, continent):
    mongo_uri = get_mongo_uri(instance_type, continent)
    client = MongoClient(mongo_uri)
    return client

# グローバル変数 municipalities_data の初期化
municipalities_data = {}

# municipalities.json の読み込み
municipalities_file_path = os.path.join(BASE_DIR, 'municipalities.json')

# ファイルが存在するか確認
if not os.path.exists(municipalities_file_path):
    raise FileNotFoundError(f"Error: '{municipalities_file_path}' not found. Please check the path.")

# ファイルを開くときにエラーハンドリングを追加
try:
    with open(municipalities_file_path, 'r', encoding='utf-8') as file:
        municipalities_data = json.load(file)
except json.JSONDecodeError as e:
    raise ValueError(f"Error decoding JSON from '{municipalities_file_path}': {e}")
except Exception as e:
    raise RuntimeError(f"An error occurred while reading '{municipalities_file_path}': {e}")

def determine_receiver_continent(receiver_municipality):
    try:
        # 'continent-city' の形式から大陸名を抽出
        continent = receiver_municipality.split('-')[0]
        return continent
    except IndexError:
        return "Default"

def determine_municipal_chain(sender_municipality, receiver_municipality):
    global municipalities_data  # グローバル変数を参照する

    # 市町村の URL を保存するための辞書を初期化
    municipal_chain_urls = {}

    # 大陸ごとの市町村情報をループしてURLを設定
    for continent, continent_data in municipalities_data.items():
        for city_data in continent_data.get('cities', []):
            # cities データから各市町村のURLを構築
            city_name = city_data['name']
            city_port = city_data['city_flask_port']  # 修正: 'city_port' から 'city_flask_port'へ
            municipal_chain_urls[f"{continent}-{city_name}"] = f"http://127.0.0.1:{city_port}"

    # 送信者または受信者の市町村に基づいて Municipal Chain の URL を選択
    if sender_municipality in municipal_chain_urls:
        return municipal_chain_urls[sender_municipality]
    elif receiver_municipality in municipal_chain_urls:
        return municipal_chain_urls[receiver_municipality]
    else:
        # デフォルトの city_flask_port を使用する
        default_city_port = municipalities_data['Default']['cities'][0]['city_flask_port']
        return f"http://127.0.0.1:{default_city_port}"  # その他の市町村チェーン（デフォルト）

@app.route('/login', methods=['POST'])
def login():
    data = request.get_json()
    username = data.get('username')
    password = data.get('password')

    # ユーザー認証の簡単な例
    if username in users and users[username] == password:
        # JWTを発行
        token = jwt.encode({
            'username': username,
            'exp': datetime.datetime.utcnow() + datetime.timedelta(hours=1)  # 1時間有効
        }, JWT_SECRET, algorithm='HS256')
        
        return jsonify({'token': token})
    else:
        return jsonify({'message': 'Invalid credentials'}), 401
    
@app.route('/')
def serve_index():
    if app.static_folder is not None:
        return send_from_directory(app.static_folder, 'index.html')
    else:
        return "Static folder is not set", 404

@app.route('/api/municipalities', methods=['GET'])
def get_municipalities():
    try:
        municipalities = []
        for continent, data in municipalities_data.items():
            for city in data.get('cities', []):
                city_id = f"{continent}-{city['name']}"
                municipalities.append({
                    "id": city_id,
                    "name": city['name'],
                    "continent": continent
                })
        return jsonify(municipalities), 200
    except Exception as e:
        print(f"Error fetching municipalities: {e}")
        return jsonify({'message': 'Internal server error.'}), 500

@app.route('/api/receivers', methods=['GET'])
def get_receivers():
    try:
        # すべての大陸を対象にする
        receivers_set = set()
        for continent in municipalities_data.keys():
            # MongoDBに接続
            mongo_client = get_mongo_connection("send_pending", continent)
            mongo_collection = mongo_client['transactions_db']['transactions']

            # `status` が 'send_pending' のトランザクションから受信者を取得
            transactions_cursor = mongo_collection.find({'status': 'send_pending'})
            for txn in transactions_cursor:
                receivers_set.add(txn['receiver'])

        # 受信者名のリストを作成
        receivers = [{'name': receiver} for receiver in receivers_set]
        return jsonify(receivers), 200
    except Exception as e:
        print(f"Error fetching receivers: {e}")
        return jsonify({'message': 'Internal server error.'}), 500

@app.route('/api/pending_transactions', methods=['GET'])
def get_pending_transactions():
    try:
        # クエリパラメータから受信者の識別子を取得
        receiver = request.args.get('receiver')
        receiver_municipality = request.args.get('receiver_municipality')
        if not receiver or not receiver_municipality:
            return jsonify({'message': 'Receiver identifier and receiver municipality are required.'}), 400

        # 大陸情報を抽出
        receiver_continent = determine_receiver_continent(receiver_municipality)

        # MongoDBに接続
        mongo_client = get_mongo_connection("send_pending", receiver_continent)
        mongo_collection = mongo_client['transactions_db']['transactions']

        # クエリを実行
        transactions_cursor = mongo_collection.find({
            'receiver': receiver,
            'receiver_municipal_id': receiver_municipality,
            'status': 'send_pending'
        })

        transactions = []
        for txn in transactions_cursor:
            txn['_id'] = str(txn['_id'])
            # datetimeオブジェクトを文字列に変換
            if 'created_at' in txn and isinstance(txn['created_at'], datetime):
                txn['created_at'] = txn['created_at'].isoformat()
            if 'updated_at' in txn and isinstance(txn['updated_at'], datetime):
                txn['updated_at'] = txn['updated_at'].isoformat()
            transactions.append(txn)

        return jsonify(transactions), 200

    except Exception as e:
        print(f"Error fetching pending transactions: {e}")
        return jsonify({'message': 'Internal server error.'}), 500

@app.route('/api/receive_transaction', methods=['POST'])
def receive_transaction():
    data = request.get_json()
    transaction_id = data.get('transaction_id')
    receiver_municipality = data.get('receiver_municipality')

    if not transaction_id or not receiver_municipality:
        return jsonify({'message': 'transaction_id and receiver_municipality are required.'}), 400

    try:
        # 大陸情報を抽出
        receiver_continent = determine_receiver_continent(receiver_municipality)

        # MongoDBに接続
        mongo_client = get_mongo_connection("send_pending", receiver_continent)
        mongo_collection = mongo_client['transactions_db']['transactions']

        # トランザクションを取得
        transaction = mongo_collection.find_one({
            'transaction_id': transaction_id,
            'receiver_municipal_id': receiver_municipality
        })

        if not transaction:
            return jsonify({'message': 'Transaction not found.'}), 404

        # トランザクションのステータスを 'receive_pending' に更新
        result = mongo_collection.update_one(
            {'transaction_id': transaction_id, 'receiver_municipal_id': receiver_municipality},
            {'$set': {'status': 'receive', 'updated_at': datetime.now(timezone.utc)}}
        )

        if result.modified_count == 0:
            return jsonify({'message': 'Failed to update transaction status.'}), 500

        # BSONドキュメントをJSONに変換
        transaction_json = json.loads(json_util.dumps(transaction))

        # '_id' フィールドを削除
        transaction_json.pop('_id', None)

        # Municipal Chain の URL を決定
        municipal_chain_url = determine_municipal_chain(transaction_json['sender_municipality'], transaction_json['receiver_municipality'])

        # トランザクションを Municipal Chain に送信
        response = requests.post(f'{municipal_chain_url}/receive_transaction', json=transaction_json)

        if response.status_code in [200, 202]:
            # トランザクションのステータスを 'complete' に更新
            result = mongo_collection.update_one(
                {'transaction_id': transaction_id, 'receiver_municipal_id': receiver_municipality},
                {'$set': {'status': 'complete', 'updated_at': datetime.now(timezone.utc)}}
            )
            if result.modified_count == 0:
                return jsonify({'message': 'Failed to update transaction status to complete.'}), 500

            print(f"Transaction {transaction_id} status updated to 'complete'.")

            # ステータスが 'complete' になったので、トランザクションを分析用データベースに移行
            try:
                analytics_uri = get_mongo_uri("analytics", receiver_continent)
                analytics_client = MongoClient(analytics_uri)
                analytics_collection = analytics_client['analytics_db']['transactions']

                # トランザクションを挿入
                analytics_collection.insert_one(transaction)
                print(f"Transaction {transaction_id} migrated to analytics database.")

                # オペレーショナルデータベースからトランザクションを削除
                mongo_collection.delete_one({'transaction_id': transaction_id, 'receiver_municipal_id': receiver_municipality})
            except Exception as e:
                print(f"Failed to migrate transaction: {e}")
                return jsonify({'message': 'Failed to migrate transaction to analytics database.'}), 500

            return jsonify({'message': f'Transaction {transaction_id} received and processed successfully.'}), 200
        else:
            try:
                error_msg = response.json().get('message', 'Failed to process transaction.')
            except json.JSONDecodeError:
                error_msg = 'Failed to process transaction.'
            return jsonify({'message': error_msg}), response.status_code
    except Exception as e:
        print(f"Error receiving transaction: {e}")
        return jsonify({'message': 'Internal server error.'}), 500

## トランザクションの拒否機能（オプション）
@app.route('/api/reject_transaction', methods=['POST'])
def reject_transaction():
    data = request.get_json()
    transaction_id = data.get('transaction_id')
    receiver_municipality = data.get('receiver_municipality')

    if not transaction_id or not receiver_municipality:
        return jsonify({'message': 'transaction_id and receiver_municipality are required.'}), 400

    try:
        receiver_continent = determine_receiver_continent(receiver_municipality)
        mongo_client = get_mongo_connection("send_pending", receiver_continent)
        mongo_collection = mongo_client['transactions_db']['transactions']

        result = mongo_collection.update_one(
            {'transaction_id': transaction_id, 'receiver_municipal_id': receiver_municipality},
            {'$set': {'status': 'rejected', 'updated_at': datetime.now(timezone.utc)}}
        )

        if result.modified_count == 0:
            return jsonify({'message': 'Failed to update transaction status.'}), 500

        return jsonify({'message': f'Transaction {transaction_id} rejected successfully.'}), 200
    except Exception as e:
        print(f"Error rejecting transaction: {e}")
        return jsonify({'message': 'Internal server error.'}), 500

if __name__ == '__main__':
    # 環境変数またはデフォルト値から大陸名を取得
    current_continent = os.getenv('CURRENT_CONTINENT', 'Default')

    # Flask ポートを取得
    flask_port = municipalities_data.get(current_continent, {}).get('flask_port', 5000)
    try:
        flask_port = int(flask_port)
    except ValueError:
        print(f"Invalid port number '{flask_port}' for continent '{current_continent}'. Using default port 5000.")
        flask_port = 5000

    # アプリケーションを起動
    app.run(host='0.0.0.0', port=flask_port, debug=True)
