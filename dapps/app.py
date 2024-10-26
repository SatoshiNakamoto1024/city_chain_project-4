from flask import Flask, request, jsonify
from flask_cors import CORS  # ここでCORSをインポート
from datetime import datetime, timedelta, timezone
import threading
import time
from pymongo import MongoClient
import sys
import os
import requests
import json
import base64  # これを追加
import uuid  # ここでuuidをインポート
from ntru import Ntru  # 相対パスの場合
from random import choice
import hashlib
import pytz

# 現在のディレクトリ（dappsディレクトリ）をパスに追加
sys.path.append(os.path.dirname(__file__))

# グローバル変数で送信履歴を保持
recent_transactions = []

app = Flask(__name__)
CORS(app, resources={r"/*": {"origins": "*"}})
ntru = Ntru()
app.config['JWT_SECRET_KEY'] = 'your_jwt_secret_key'  # セキュリティ上、長くて推測しにくいキーを設定してください
jwt = JWTManager(app)

# ログイン用のエンドポイント
@app.route('/api/login', methods=['POST'])
def login():
    data = request.get_json()
    username = data.get('username')
    password = data.get('password')

    # ここでユーザーの認証を行う（例: データベースと照合）
    if username == 'testuser' and password == 'testpass':
        access_token = create_access_token(identity=username, expires_delta=datetime.timedelta(hours=1))
        return jsonify(token=access_token), 200
    else:
        return jsonify(message='Invalid credentials'), 401

def load_individuals_data():
    # base_dir のみを取得し、dapps ディレクトリを重複させないように修正
    base_dir = os.path.dirname(os.path.abspath(__file__))
    file_path = os.path.join(base_dir, 'indivisuals.json')  # 'dapps' を追加しない

    if not os.path.exists(file_path):
        raise FileNotFoundError(f"{file_path} not found. Please check the file path.")

    # ファイルを開いてデータを読み込む
    with open(file_path, 'r', encoding='utf-8') as file:
        return json.load(file)
    
# Flaskポートを取得する関数
def get_flask_port(continent_name):
    # 大陸名のflask_portを取得
    if continent_name in municipalities_data:
        flask_port = municipalities_data[continent_name].get("flask_port")
        if flask_port:
            return int(flask_port)
    
    # Defaultのflask_portを取得
    return int(municipalities_data["Default"]["flask_port"])

# MongoDB設定ファイルパス
MONGODB_CONFIG_PATH = os.path.join("D:\\city_chain_project", 'mongodb_config.json')
# MongoDB設定ファイルを読み込む
def load_mongodb_config():
    try:
        with open(MONGODB_CONFIG_PATH, 'r', encoding='utf-8') as file:
            content = file.read()
            print("MongoDB config content:", content)  # ファイルの内容を表示してデバッグ
            return json.loads(content)
    except FileNotFoundError:
        raise FileNotFoundError(f"Error: '{MONGODB_CONFIG_PATH}' not found. Please check the path.")
    except json.JSONDecodeError as e:
        raise ValueError(f"Error decoding JSON from '{MONGODB_CONFIG_PATH}': {e}")

mongodb_config = load_mongodb_config()

# ファイルパスを指定
mongodb_config_path = "mongodb_config.json"

# 確認用
print("Final MongoDB Config:", mongodb_config)

# MongoDB URI を取得する関数
def get_mongo_uri(instance_type, continent):
    if instance_type in mongodb_config and continent in mongodb_config[instance_type]:
        return mongodb_config[instance_type][continent]
    elif instance_type in mongodb_config and 'default' in mongodb_config[instance_type]:
        return mongodb_config[instance_type]['default']
    else:
        raise ValueError(f"MongoDB URI not found for instance type '{instance_type}' and continent '{continent}'")

# MongoDB コネクションを取得する
def get_mongo_connection(instance_type, continent):
    mongo_uri = get_mongo_uri(instance_type, continent)
    client = MongoClient(mongo_uri)
    return client

# Example usage: send_pending のアジア用インスタンスを取得
try:
    mongo_client = get_mongo_connection("send_pending", "Asia")
    send_pending_collection = mongo_client['transactions_db']['send_pending_transactions']
    print("Successfully connected to send_pending MongoDB instance for Asia")
except Exception as e:
    print(f"Error connecting to MongoDB: {e}")

class DPoS:
    def __init__(self, sender_municipalities, receiver_municipalities):
        self.sender_municipalities = sender_municipalities
        self.receiver_municipalities = receiver_municipalities
        self.approved_representative = None

    def elect_representative(self, sender_or_receiver):
        if sender_or_receiver == 'sender':
            self.approved_representative = choice(self.sender_municipalities)
        elif sender_or_receiver == 'receiver':
            self.approved_representative = choice(self.receiver_municipalities)
        return self.approved_representative

    def approve_transaction(self, transaction):
        if self.approved_representative:
            transaction['signature'] = f"approved_by_{self.approved_representative}"
            return True
        return False
    
class ProofOfPlace:
    def __init__(self, location):
        self.location = location
        self.timestamp = datetime.now(pytz.utc)

    def generate_proof(self):
        hasher = hashlib.sha256()
        hasher.update(f"{self.location}{self.timestamp}".encode())
        return hasher.hexdigest()

    @staticmethod
    def verify_proof(proof, location, timestamp):
        hasher = hashlib.sha256()
        hasher.update(f"{location}{timestamp}".encode())
        return hasher.hexdigest() == proof

# 大陸名を抽出する関数（送信者用）
def determine_sender_continent(sender_municipality_data):
    try:
        # 'continent-city' の形式から大陸名を抽出
        continent = sender_municipality_data.split('-')[0]
        return continent
    except IndexError:
        return None

# 大陸名を抽出する関数（受信者用）
def determine_receiver_continent(receiver_municipality_data):
    try:
        # 'continent-city' の形式から大陸名を抽出
        continent = receiver_municipality_data.split('-')[0]
        return continent
    except IndexError:
        return None

# 送信者市町村名を抽出する関数
def determine_sender_municipality(sender_municipality_data):
    try:
        # 'continent-city' の形式から市町村名を抽出
        municipality = sender_municipality_data.split('-')[1]
        return municipality
    except IndexError:
        return "Unknown"  # フォーマットが異常な場合の対処

# 受信者市町村名を抽出する関数
def determine_receiver_municipality(receiver_municipality_data):
    try:
        # 'continent-city' の形式から市町村名を抽出
        municipality = receiver_municipality_data.split('-')[1]
        return municipality
    except IndexError:
        return "Unknown"  # フォーマットが異常な場合の対処

# 送信トランザクションの制限を確認する関数（1分以内に10件まで）
def check_recent_transactions():
    global recent_transactions
    current_time = datetime.now(timezone.utc)
    # 1分以内のトランザクションをフィルター
    one_minute_ago = current_time - timedelta(minutes=1)
    recent_transactions = [t for t in recent_transactions if t >= one_minute_ago]

    # 現在のトランザクション数が10件を超えていればエラー
    if len(recent_transactions) >= 10:
        return False
    return True

@app.route('/process', methods=['POST'])
def process_transaction():
    data = request.json

    # データがNoneの場合、エラーを返す
    if data is None:
        return jsonify({"error": "No data received"}), 400
    
    # 入力データの検証
    required_fields = ['sender', 'sender_municipality', 'receiver', 'receiver_municipality', 'amount']
    for field in required_fields:
        if field not in data or data[field] is None:
            return jsonify({"error": f"Missing or invalid field: {field}"}), 400
    
    # 市町村のデータを取得
    sender_municipalities = [data['sender_municipality']]
    receiver_municipalities = [data['receiver_municipality']]
    
    # 大陸情報を抽出する
    sender_continent = determine_sender_continent(data['sender_municipality'])
    receiver_continent = determine_receiver_continent(data['receiver_municipality'])

    # データに追加
    data['sender_continent'] = sender_continent
    data['receiver_continent'] = receiver_continent
    dpos = DPoS(sender_municipalities, receiver_municipalities)
    
    # 送信者と受信者の代表者を選出
    sender_rep = dpos.elect_representative('sender')
    receiver_rep = dpos.elect_representative('receiver')
    
    # トランザクションの承認
    transaction = {
        'sender': data['sender'],
        'receiver': data['receiver'],
        'amount': data['amount'],
        'sender_municipality': data['sender_municipality'],
        'receiver_municipality': data['receiver_municipality'],
        'sender_continent': data['sender_continent'],
        'receiver_continent': data['receiver_continent'],
    }
    
    if dpos.approve_transaction(transaction):
        return jsonify({"status": "Transaction approved", "transaction": transaction}), 200
    else:
        return jsonify({"error": "Failed to approve transaction"}), 500

# ProofOfPlaceの生成と検証を動的に行う関数
def generate_and_verify_proof(location):
    # 動的に現在のタイムスタンプを使ってProofOfPlaceを生成
    proof_of_place = ProofOfPlace(location=location)
    generated_proof = proof_of_place.generate_proof()

    # タイムスタンプと場所を使って生成された証明を検証
    is_valid = ProofOfPlace.verify_proof(generated_proof, location, proof_of_place.timestamp)

    print(f"Generated proof: {generated_proof}")
    print(f"Proof is valid: {is_valid}")
    return generated_proof, is_valid
    
@app.route('/create_transaction', methods=['POST'])
def create_transaction():
    data = request.get_json()  # リクエストデータを取得
    
    # リクエストデータが存在しない場合
    if not data:
        return jsonify({"error": "Invalid JSON or no data received"}), 400

    # 必要なフィールドを確認
    required_fields = ['sender', 'receiver', 'amount', 'sender_municipality', 'receiver_municipality', 'continent']
    for field in required_fields:
        if field not in data:
            return jsonify({"error": f"Missing field: {field}"}), 400

    # 大陸情報を抽出
    sender_continent = determine_sender_continent(data['sender_municipality'])
    receiver_continent = determine_receiver_continent(data['receiver_municipality'])

    # データの整形
    transaction = {
        "sender": data.get('sender', ''),
        "receiver": data.get('receiver', ''),
        "amount": float(data.get('amount', 0)),
        "timestamp": datetime.now(timezone.utc).isoformat() + "Z",
        "transaction_id": str(uuid.uuid4()),
        "verifiable_credential": data.get('verifiable_credential', ''),
        "signature": data.get('signature', 'dummy_signature'),
        "subject": data.get('subject', ''),
        "action_level": data.get('action_level', ''),
        "dimension": data.get('dimension', ''),
        "fluctuation": data.get('fluctuation', ''),
        "organism_name": data.get('organism_name', ''),
        "sender_municipality": data.get('sender_municipality', ''),
        "receiver_municipality": data.get('receiver_municipality', ''),
        "sender_municipal_id": data.get('sender_municipality', ''),  
        "receiver_municipal_id": data.get('receiver_municipality', ''),  
        "details": data.get('details', ''),
        "goods_or_money": data.get('goods_or_money', ''),
        "transaction_type": data.get('transaction_type', 'send'),  # クライアントから送信された値を使用
        "location": data.get('location', ''),
        "proof_of_place": data.get('proof_of_place', ''),
        "status": "send_pending",  # 初期ステータスを設定
        "created_at": datetime.now(timezone.utc).isoformat() + "Z",  # 作成日時を追加
    }

    # DPoSのインスタンスを作成し、送信者と受信者の代表者を選出
    sender_municipalities = [data.get('sender_municipality', '')]
    receiver_municipalities = [data.get('receiver_municipality', '')]
    dpos = DPoS(sender_municipalities, receiver_municipalities)
    sender_rep = dpos.elect_representative('sender')
    receiver_rep = dpos.elect_representative('receiver')

    # トランザクションの承認
    if dpos.approve_transaction(transaction):
        # MongoDBコレクションに接続
        mongo_uri = get_mongo_uri("send_pending", transaction['sender_continent'])
        mongo_client = MongoClient(mongo_uri)
        mongo_collection = mongo_client['transactions_db']['transactions']

        # トランザクションを挿入
        try:
            mongo_collection.insert_one(transaction)
            print("Transaction inserted successfully with status 'send_pending'")
        except Exception as e:
            print(f"Failed to insert transaction: {e}")
            return jsonify({"error": "Failed to insert transaction"}), 500

        # 送信先のMunicipal Chainにトランザクションを送信
        municipal_chain_url = determine_municipal_chain(transaction['sender_municipality'], transaction['receiver_municipality'])
        print(f"Sending to Municipal Chain URL: {municipal_chain_url}")

        try:
            response = requests.post(f'{municipal_chain_url}/receive_transaction', json=transaction)

            if response.status_code == 200:
                print("Transaction sent successfully to Municipal Chain")
                return jsonify({"status": "success"}), 200
            else:
                print(f"Failed to send transaction: {response.text}")
                return jsonify({"error": "Failed to send transaction"}), response.status_code

        except requests.exceptions.RequestException as e:
            print(f"Error occurred: {str(e)}")
            return jsonify({"error": str(e)}), 500  # エラー時の処理
    else:
        return jsonify({"error": "Failed to approve transaction"}), 500

@app.route('/add_block', methods=['POST'])
def add_block():
    data = request.json

    # data が None かどうかを確認
    if data is None:
        return jsonify({"error": "No data received"}), 400

    # 送信者と受信者の市町村情報を取得
    sender_municipality = data.get('sender_municipality', '')
    receiver_municipality = data.get('receiver_municipality', '')

    # 市町村に基づいて動的に Municipal Chain の URL を決定
    municipal_chain_url = determine_municipal_chain(sender_municipality, receiver_municipality)

    # Municipal Chain にデータを送信
    try:
        response = requests.post(f'{municipal_chain_url}/add_block', json=data)
        response.raise_for_status()  # HTTPエラーステータスコードをチェック

        # レスポンスを JSON 形式で返す
        return jsonify(response.json())
    except requests.exceptions.RequestException as e:
        # エラーメッセージを出力してエラーレスポンスを返す
        print(f"Error occurred while sending data to Municipal Chain: {str(e)}")
        return jsonify({"error": "Failed to send data to Municipal Chain"}), 500

# BASE_DIR の定義
BASE_DIR = os.path.dirname(os.path.abspath(__file__))

# グローバル変数 municipalities_data の初期化
municipalities_data = {}

# municipalities_data.json の読み込み
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

# municipalities_data をグローバルに設定
def initialize_globals():
    global municipalities_data
    return municipalities_data

# 関数内でグローバル変数を使う
def determine_municipal_chain(sender_municipality, receiver_municipality):
    global municipalities_data  # グローバル変数を参照する

    # 市町村の URL を保存するための辞書を初期化
    municipal_chain_urls = {}

    # 大陸ごとの市町村情報をループしてURLを設定
    for continent, continent_data in municipalities_data.items():
        for city_data in continent_data.get('cities', []):
            # cities データから各市町村のURLを構築
            city_name = city_data['name']
            city_port = city_data['city_port']
            municipal_chain_urls[f"{continent}-{city_name}"] = f"http://127.0.0.1:{city_port}"

    # 送信者または受信者の市町村に基づいて Municipal Chain の URL を選択
    if sender_municipality in municipal_chain_urls:
        return municipal_chain_urls[sender_municipality]
    elif receiver_municipality in municipal_chain_urls:
        return municipal_chain_urls[receiver_municipality]
    else:
        # Default の city_port を使用する
        default_city_port = municipalities_data['Default']['cities'][0]['city_port']
        return f"http://127.0.0.1:{default_city_port}"  # その他の市町村チェーン（デフォルト）


def determine_continental_chain_url(continent, municipality):
    global municipalities_data  # グローバル変数を参照する

    # 大陸ごとのContinental Main ChainのベースURLを取得する
    continent_data = municipalities_data.get(continent, {})
    
    # 大陸のflask_portを取得する
    base_flask_port = continent_data.get('flask_port', municipalities_data['Default']['flask_port'])

    # 市町村に基づいて完全なURLを構築
    for city_data in continent_data.get('cities', []):
        if city_data['name'] == municipality:
            # city_flask_portを使ってURLを構築
            city_flask_port = city_data.get('city_flask_port', base_flask_port)
            return f"http://localhost:{city_flask_port}/continental_main_chain_endpoint"
    
    # 市町村が見つからなかった場合、デフォルトのflask_portを利用してURLを返す
    return f"http://localhost:{base_flask_port}/continental_main_chain_endpoint"

@app.route('/sign_transaction', methods=['POST'])
def sign_transaction():
    try:
        data = request.get_json(force=True)  # JSONデータを強制的に取得
        transaction = json.dumps(data, ensure_ascii=False)  # 日本語を含めたデータを正しくエンコード
        signature = ntru.sign(transaction.encode('utf-8'), data['private_key'])  # UTF-8でエンコード
        return jsonify({"signature": signature.hex()})  # 署名を16進数に変換して返す
    except Exception as e:
        return jsonify({"error": str(e)}), 400

@app.route('/verify_signature', methods=['POST'])
def verify_signature():
    try:
        data = request.get_json(force=True)  # 入力データをJSON形式として強制的に解釈
        transaction = json.dumps(data['transaction'], ensure_ascii=False)  # 日本語を扱うためにensure_ascii=Falseを設定
        signature = bytes.fromhex(data['signature'])  # 署名を16進数からバイト列に変換
        public_key = data['public_key']
        is_valid = ntru.verify(transaction.encode('utf-8'), signature, public_key)  # UTF-8エンコーディングでエンコード
        return jsonify({"is_valid": is_valid})
    except Exception as e:
        return jsonify({"error": str(e)}), 400

@app.route('/generate_proof_of_place', methods=['POST'])
def generate_proof_of_place():
    try:
        data = request.get_json(force=True)  # JSONデータを強制的に取得
        proof_of_place = ProofOfPlace(location=(data['latitude'], data['longitude']))
        proof = proof_of_place.generate_proof()
        return jsonify({"proof": proof, "timestamp": proof_of_place.timestamp.isoformat()})
    except Exception as e:
        return jsonify({"error": str(e)}), 400

@app.route('/verify_proof_of_place', methods=['POST'])
def verify_proof_of_place():
    try:
        data = request.get_json(force=True)  # JSONデータを強制的に取得
        proof = data['proof']
        location = (data['latitude'], data['longitude'])
        timestamp = datetime.fromisoformat(data['timestamp'])
        is_valid = ProofOfPlace.verify_proof(proof, location, timestamp)
        return jsonify({"is_valid": is_valid})
    except Exception as e:
        return jsonify({"error": str(e)}), 400

# Municipal Chainへのトランザクション送信を行うエンドポイント
@app.route('/send', methods=['POST'])
def send_love_currency():
    global recent_transactions
    data = request.json  # フロントエンドからのデータを受け取る
    
    # dataがNoneの場合はエラーを返す
    if data is None:
        return jsonify({"error": "Invalid input data"}), 400
    
    # トランザクション数を確認（1分以内に10件以上ならエラー）
    if not check_recent_transactions():
        return jsonify({"error": "Too many transactions in the last minute. Please try again later."}), 429

    # 必要なフィールドが含まれているか確認
    required_fields = ['sender', 'receiver', 'amount', 'sender_municipality', 'receiver_municipality', 'continent', 'private_key', 'seed_phrase']
    for field in required_fields:
        if field not in data:
            return jsonify({"error": f"Missing field: {field}"}), 400
        
    # 大陸情報を抽出
    sender_continent = determine_sender_continent(data['sender_municipality'])
    receiver_continent = determine_receiver_continent(data['receiver_municipality'])
    
    # 送信者と受信者が有効か確認
    individuals_data = load_individuals_data()  # indivisuals.json からデータをロードする
    valid_participants = {ind['name'] for ind in individuals_data}
    if data['sender'] not in valid_participants or data['receiver'] not in valid_participants:
        return jsonify({"error": "Invalid sender or receiver"}), 400
    
    private_key = data['private_key']  # 秘密鍵を取得
    seed_phrase = data['seed_phrase']  # シードフレーズを取得
    
    # amount を float に変換
    try:
        data['amount'] = float(data['amount'])
    except ValueError:
        return jsonify({"error": "Invalid amount"}), 400
    
    # 必要なデータを追加
    data['verifiable_credential'] = 'VerifiableCredentialData'
    data['signature'] = base64.b64encode(b"dummy_signature").decode('utf-8')
    data['location'] = 'LocationData'
    data['timestamp'] = datetime.now(timezone.utc).isoformat() + "Z"
    data['proof_of_place'] = 'ProofOfPlaceData'
    data['transaction_id'] = str(uuid.uuid4())
    data['transaction_type'] = 'send'  # transaction_type を追加
    data['sender_continent'] = sender_continent  # 修正
    data['receiver_continent'] = receiver_continent  # 修正
    data['status'] = 'send'  # トランザクションの初期ステータスを「send」に設定
    data['created_at'] = datetime.now(timezone.utc).isoformat() + "Z"
    data['sender_municipal_id'] = data['sender_municipality']
    data['receiver_municipal_id'] = data['receiver_municipality']

    print("Received data:", data)  # デバッグ用に受け取ったデータをコンソールに出力

     # 市町村名に基づいて Municipal Chain の URL を取得
    municipal_chain_url = determine_municipal_chain(data['sender_municipality'], data['receiver_municipality'])
    print(f"Sending to Municipal Chain URL: {municipal_chain_url}")

    # Municipal Chainのエンドポイントにトランザクションを送信
    # トランザクションを送信する前に暗号化処理を追加
    try:
        transaction_str = json.dumps(data, sort_keys=True)  # トランザクションデータをJSON文字列に変換
        signature = ntru.sign(transaction_str.encode('utf-8'), private_key)  # NTRUで署名
        data['signature'] = base64.b64encode(signature).decode('utf-8')  # 署名をBase64でエンコードして格納
    except Exception as e:
        return jsonify({"error": f"Signature creation failed: {str(e)}"}), 500

    # トランザクションを MongoDB に保存、ステータスは "send" のまま
    try:
        mongo_uri = get_mongo_uri("send", data['sender_continent'])  # "send" 状態として保存
        mongo_client = MongoClient(mongo_uri)
        mongo_collection = mongo_client['transactions_db']['transactions']
        mongo_collection.insert_one(data)
        print("Transaction inserted successfully with status 'send'")
    except Exception as e:
        print(f"Failed to insert transaction: {e}")
        return jsonify({"error": "Failed to insert transaction"}), 500

    # Municipal Chainにトランザクションを送信
    try:
        response = requests.post(f'{municipal_chain_url}/receive_transaction', json=data)
        if response.status_code == 200:
            print("Transaction sent successfully to Municipal Chain")
            
            # 承認されたらステータスを "send_pending" に更新
            mongo_collection.update_one(
                {"transaction_id": data['transaction_id']},
                {"$set": {"status": "send_pending"}}
            )
            return jsonify({"status": "success"}), 200
        else:
            print(f"Failed to send transaction: {response.text}")
            return jsonify({"error": "Failed to send transaction"}), response.status_code
    except requests.exceptions.RequestException as e:
        return jsonify({"error": str(e)}), 500

# ルートエンドポイント
@app.route('/')
def index():
    return "Flask app is running!"

def home():
    return "Flask app is running!"

@app.route('/update_status', methods=['POST'])
def update_status():
    data = request.get_json()
    
    if data is None:
        return jsonify({"error": "No data received"}), 400
    
    # 必要なフィールドを確認
    required_fields = ['transaction_id', 'new_status', 'sender_municipal_id']
    for field in required_fields:
        if field not in data:
            return jsonify({"error": f"Missing field: {field}"}), 400
    
    transaction_id = data['transaction_id']
    new_status = data['new_status']
    sender_municipal_id = data['sender_municipal_id']
    
    # sender_municipal_id から大陸名を取得
    sender_continent = determine_sender_continent(sender_municipal_id)
    if not sender_continent:
        sender_continent = 'Default'
    
    try:
        # MongoDBコレクションに接続（send_pending）
        mongo_uri = get_mongo_uri("send_pending", sender_continent)
        mongo_client = MongoClient(mongo_uri)
        mongo_collection = mongo_client['transactions_db']['transactions']
        
        # トランザクションを更新
        result = mongo_collection.update_one(
            {"transaction_id": transaction_id, "sender_municipal_id": sender_municipal_id},
            {"$set": {"status": new_status, "updated_at": datetime.now(timezone.utc).isoformat() + "Z"}}
        )
        
        if result.matched_count == 0:
            return jsonify({"error": "Transaction not found"}), 404
        
        print(f"Transaction {transaction_id} updated to status {new_status}")
        
        # ステータスが "complete" の場合、分析用データベースに移行
        if new_status == "complete":
            transaction = mongo_collection.find_one({"transaction_id": transaction_id})
            if transaction:
                # 分析用 MongoDB に接続
                analytics_uri = get_mongo_uri("analytics", sender_continent)
                analytics_client = MongoClient(analytics_uri)
                analytics_collection = analytics_client['analytics_db']['transactions']
                
                # トランザクションを挿入
                analytics_collection.insert_one(transaction)
                print(f"Transaction {transaction_id} migrated to analytics database.")
                
                # 元のトランザクションを削除
                mongo_collection.delete_one({"transaction_id": transaction_id})
                
        return jsonify({"status": "Transaction updated"}), 200
    except Exception as e:
        print(f"Failed to update transaction: {e}")
        return jsonify({"error": "Failed to update transaction"}), 500

@app.route('/api/send_transaction', methods=['POST'])
@jwt_required()  # トークン認証が必要
def send_transaction():
    current_user = get_jwt_identity()  # 現在の認証済みユーザーを取得
    data = request.get_json()  # リクエストデータをJSON形式で取得

    # 必要なフィールドを確認
    required_fields = ['sender', 'receiver', 'amount', 'sender_municipality', 'receiver_municipality', 'private_key', 'seed_phrase']
    for field in required_fields:
        if field not in data or data[field] is None:
            return jsonify({"error": f"Missing or invalid field: {field}"}), 400

    # 市町村のデータを取得
    sender_municipalities = [data['sender_municipality']]
    receiver_municipalities = [data['receiver_municipality']]

    # 大陸情報を抽出
    sender_continent = determine_sender_continent(data['sender_municipality'])
    receiver_continent = determine_receiver_continent(data['receiver_municipality'])

    # トランザクションデータを整形
    transaction = {
        "sender": data['sender'],
        "receiver": data['receiver'],
        "amount": float(data['amount']),
        "timestamp": datetime.now(timezone.utc).isoformat() + "Z",
        "transaction_id": str(uuid.uuid4()),
        "verifiable_credential": data.get('verifiable_credential', ''),
        "signature": data.get('signature', ''),
        "subject": data.get('subject', ''),
        "action_level": data.get('action_level', ''),
        "dimension": data.get('dimension', ''),
        "fluctuation": data.get('fluctuation', ''),
        "organism_name": data.get('organism_name', ''),
        "sender_municipality": data['sender_municipality'],
        "receiver_municipality": data['receiver_municipality'],
        "sender_continent": sender_continent,
        "receiver_continent": receiver_continent,
        "transaction_type": 'send',
        "status": "send_pending",  # トランザクションの初期ステータスを設定
        "created_at": datetime.now(timezone.utc).isoformat() + "Z",
        "sender_municipal_id": data['sender_municipality'],
        "receiver_municipal_id": data['receiver_municipality']
    }

    # 送信トランザクションの代表者を選出（DPoS）
    dpos = DPoS(sender_municipalities, receiver_municipalities)
    sender_rep = dpos.elect_representative('sender')
    receiver_rep = dpos.elect_representative('receiver')

    # トランザクションの承認
    if dpos.approve_transaction(transaction):
        try:
            mongo_uri = get_mongo_uri("send_pending", transaction['sender_continent'])
            mongo_client = MongoClient(mongo_uri)
            mongo_collection = mongo_client['transactions_db']['transactions']
            mongo_collection.insert_one(transaction)
            print("Transaction inserted successfully with status 'send_pending'")
        except Exception as e:
            print(f"Failed to insert transaction: {e}")
            return jsonify({"error": "Failed to insert transaction"}), 500

        municipal_chain_url = determine_municipal_chain(transaction['sender_municipality'], transaction['receiver_municipality'])
        print(f"Sending to Municipal Chain URL: {municipal_chain_url}")

        try:
            response = requests.post(f'{municipal_chain_url}/receive_transaction', json=transaction)
            if response.status_code == 200:
                print("Transaction sent successfully to Municipal Chain")
                return jsonify({"status": "success"}), 200
            else:
                print(f"Failed to send transaction: {response.text}")
                return jsonify({"error": "Failed to send transaction"}), response.status_code
        except requests.exceptions.RequestException as e:
            print(f"Error occurred: {str(e)}")
            return jsonify({"error": str(e)}), 500
    else:
        return jsonify({"error": "Failed to approve transaction"}), 500

# トランザクション生成と未完了トランザクション管理のための関数
def send_to_municipal_chain(transaction, municipal_chain_url):
    try:
        response = requests.post(f'{municipal_chain_url}/receive_transaction', json=transaction)
        if response.status_code == 200:
            print("Transaction sent successfully to Municipal Chain")
            # Municipal Chainに送信が成功したら、Continental Main Chainに未完了トランザクションを送信
            send_to_continental_main_chain(transaction)
            return response.json()
        else:
            print(f"Failed to send transaction: {response.text}")
            return None
    except requests.exceptions.RequestException as e:
        print(f"Error occurred: {str(e)}")
        return None

def send_to_continental_main_chain(transaction):
    continental_chain_url = determine_continental_chain_url(transaction['continent'], transaction['municipality'])
    
    try:
        response = requests.post(f'{continental_chain_url}/pending_transaction', json=transaction)
        if response.status_code == 200:
            print("Transaction successfully sent to Continental Main Chain")
            return response.json()
        else:
            print(f"Failed to send to Continental Main Chain: {response.text}")
            return None
    except requests.exceptions.RequestException as e:
        print(f"Error occurred when sending to Continental Main Chain: {str(e)}")
        return None

def create_indexes(instance_type, continent):
    mongo_uri = get_mongo_uri(instance_type, continent)
    mongo_client = MongoClient(mongo_uri)
    mongo_collection = mongo_client['transactions_db']['transactions']
    
    mongo_collection.create_index([("status", 1)])
    mongo_collection.create_index([("created_at", 1)])
    print(f"Indexes created on 'status' and 'created_at' fields for {instance_type} in {continent}.")
    
# 呼び出し時
create_indexes("send_pending", "Asia")

def clean_expired_send_pending_transactions(continent):
    while True:
        try:
            mongo_uri = get_mongo_uri("send_pending", continent)
            mongo_client = MongoClient(mongo_uri)
            mongo_collection = mongo_client['transactions_db']['transactions']
            
            expiration_threshold = datetime.now(timezone.utc) - timedelta(days=6*30)
            
            result = mongo_collection.delete_many({
                "status": "send_pending",
                "created_at": {"$lt": expiration_threshold.isoformat() + "Z"},
                "sender_municipal_id": {"$exists": True}
            })

            print(f"Deleted {result.deleted_count} expired send_pending transactions for {continent}.")
        except Exception as e:
            print(f"Error during cleanup for {continent}: {e}")

        time.sleep(24 * 60 * 60)  # 24時間

# アプリケーション開始時にクリーンアップタスクを開始
cleanup_thread = threading.Thread(target=clean_expired_send_pending_transactions, args=("Asia",), daemon=True)
cleanup_thread.start()

# 分析用MongoDBのURIを取得
try:
    current_continent = os.getenv('CURRENT_CONTINENT', 'Asia')  # 環境変数またはデフォルト値を使用
    ANALYTICS_MONGO_URI = get_mongo_uri("analytics", current_continent)
    analytics_client = MongoClient(ANALYTICS_MONGO_URI)
    analytics_collection = analytics_client['analytics_db']['transactions']
    print(f"Successfully connected to analytics MongoDB instance for {current_continent}")
except Exception as e:
    print(f"Error connecting to analytics MongoDB: {e}")

def migrate_to_analytics(transaction, continent):
    try:
        mongo_uri = get_mongo_uri("analytics", continent)
        analytics_client = MongoClient(mongo_uri)
        analytics_collection = analytics_client['analytics_db']['transactions']
        
        # 分析用データベースにトランザクションをコピー
        analytics_collection.insert_one(transaction)
        print(f"Transaction {transaction['transaction_id']} migrated to analytics database.")
    except Exception as e:
        print(f"Failed to migrate transaction {transaction['transaction_id']}: {e}")

if __name__ == '__main__':
    def get_flask_port(continent_name):
        # 大陸名のflask_portを取得
        if continent_name in municipalities_data:
            flask_port = municipalities_data[continent_name].get("flask_port")
            if flask_port:
                return int(flask_port)
        
        # Defaultのflask_portを取得
        return int(municipalities_data["Default"]["flask_port"])

    # ここで動的に大陸名を取得する
    current_continent = os.getenv('CURRENT_CONTINENT', 'Default')
    flask_port = get_flask_port(current_continent)
    
    # インデックスを作成
    create_indexes("send_pending", current_continent)
    
    # クリーンアップタスクを開始
    cleanup_thread = threading.Thread(target=clean_expired_send_pending_transactions, args=(current_continent,), daemon=True)
    cleanup_thread.start()
    
    app.run(host='0.0.0.0', port=flask_port)




