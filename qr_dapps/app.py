import re
from flask import Flask, request, jsonify, redirect, url_for
import openai
import os
import qrcode
from pymongo import MongoClient
from bson.objectid import ObjectId
from bson import json_util
from datetime import datetime, timezone, timedelta
import threading
import requests
import uuid
import json

app = Flask(__name__)

# MongoDB設定ファイルのパス
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
MONGODB_CONFIG_PATH = os.path.join(BASE_DIR, 'mongodb_config.json')

# MongoDB設定：mongodb_config.jsonを読み込み
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

def get_db_from_uri(uri):
    return uri.rsplit('/', 1)[-1]

# トランザクション用MongoDB設定（Defaultを使用）
transactions_mongo_uri = mongodb_config['send']['Default']
transactions_client = MongoClient(transactions_mongo_uri)
transactions_db_name = get_db_from_uri(transactions_mongo_uri)
transactions_db = transactions_client[transactions_db_name]
transactions_collection = transactions_db['transactions']

# 仕訳データ用MongoDB設定（analyticsデータベースを使用）
journal_entries_mongo_uri = mongodb_config['analytics']['Default']
journal_entries_client = MongoClient(journal_entries_mongo_uri)
journal_entries_db_name = get_db_from_uri(journal_entries_mongo_uri)
journal_entries_db = journal_entries_client[journal_entries_db_name]
journal_entries_collection = journal_entries_db['journal_entries']

# Whisper APIキー（音声入力用）
openai.api_key = os.getenv("OPENAI_API_KEY")

# 科目リスト
accounts = {
    "assets": ["愛貨", "未使用愛貨", "他者からの未収愛貨", "愛貨の投資", "前払愛貨", "他者への貸付愛貨", "共同プロジェクト出資愛貨", "他者から受け取る予定の愛貨", "愛の行動に関する知的財産", "愛貨の再評価による資産"],
    "liabilities": ["未実施行動の債務", "目標未達成による債務", "他者への未払愛貨", "愛貨の有効期限延長による債務", "他者からの借入愛貨", "愛の行動義務", "取引キャンセルによる愛貨返還義務", "前受愛貨", "他者への未使用愛貨の債務", "市町村依頼行動債務"],
    "equity": ["繰越愛貨", "愛貨評価差額", "愛貨の変動差額", "投資利益剰余愛貨", "愛貨贈与差額", "前期繰越未使用愛貨", "愛貨リザーブ", "純資産愛貨", "愛貨の振替差額", "利益剰余愛貨"],
    "revenue": ["愛の行動受取", "目標達成による愛貨の獲得", "愛貨感謝返礼収益", "受取愛貨の増加", "愛貨紹介報酬", "愛貨還元収益", "愛貨ネットワーク拡大報酬", "他者の支援による愛貨の獲得", "コミュニティ活動による愛貨の獲得", "承認成功報酬愛貨"],
    "expenses": ["愛の行動消費", "愛貨感謝返礼費用", "他者への愛貨の贈与", "目標未達成による損失", "他者への未実施行動による損失", "愛貨の有効期限切れによる損失", "愛貨の減少による費用", "行動目標達成のための愛貨使用", "他者のサポートに伴う愛貨の消費", "愛貨の再投資費用"],
    "netprofit": ["愛貨消費による貢献度（Contribution by Love Token Usage）", "他者からの愛貨受領による価値（Value from Love Tokens Received）", "愛貨の総消費（Total Love Token Usage）", "愛貨の純増減（Net Love Token Flow）"],
}

def select_revenue_account(text):
    for account in accounts['revenue']:
        if account in text:
            return account
    return "愛の行動受取"  # デフォルト収益科目

def select_asset_account(text):
    for account in accounts['assets']:
        if account in text:
            return account
    return "愛貨"  # デフォルト資産科目

def determine_journal_entries(text):
    debit_account = "未設定"
    credit_account = "未設定"

    # 愛の行動に関連する科目を収益と推測
    if any(keyword in text for keyword in ["愛の行動", "感謝", "贈与", "支援", "提供"]):
        credit_account = select_revenue_account(text)
    
    # 資産の減少として推測できる場合
    if any(keyword in text for keyword in ["資産", "減少", "使った", "消費"]):
        debit_account = select_asset_account(text)

    # 借方・貸方の科目が見つからなければ、デフォルト値を使用
    if credit_account == "未設定":
        credit_account = select_revenue_account(text)
    if debit_account == "未設定":
        debit_account = select_asset_account(text)

    return debit_account, credit_account

# テキスト解析関数（自動仕訳）
def analyze_text(text):
    result = {}

    # 送信者
    sender_match = re.search(r"送信者[は:]*\s*(\w+)", text)
    if sender_match:
        result['sender'] = sender_match.group(1)
    else:
        result['sender'] = '不明'

    # 金額
    amount_match = re.search(r"(\d+)\s*愛貨", text)
    if amount_match:
        result['amount'] = int(amount_match.group(1))
    else:
        result['amount'] = 0

    # 受信者数
    recipients_count_match = re.search(r"受信者数[は:]*\s*(\d+)", text)
    if recipients_count_match:
        result['recipients_count'] = int(recipients_count_match.group(1))
    else:
        result['recipients_count'] = 1  # デフォルトは1人

    # 借方・貸方科目の決定
    debit_account, credit_account = determine_journal_entries(text)
    result['debit'] = debit_account
    result['credit'] = credit_account

    # 詳細内容
    result['details'] = text

    # 不足情報があれば質問を促す
    missing_fields = []
    for field in ['sender', 'amount', 'debit', 'credit', 'recipients_count']:
        if field not in result or not result[field] or result[field] in ['不明', '未設定', 0]:
            missing_fields.append(field)
    
    if missing_fields:
        result['status'] = 'missing_data'
        result['missing_fields'] = missing_fields
    else:
        result['status'] = 'complete'

    return result

# 過去の仕訳データと比較して修正
def compare_and_correct_journal_entries(parsed_data):
    # 過去の仕訳を検索
    query = {
        "debit": parsed_data.get('debit'),
        "credit": parsed_data.get('credit'),
        "amount": parsed_data.get('amount')
    }
    past_entries = list(journal_entries_collection.find(query))  # MongoDBから過去の仕訳を検索

    # 過去の仕訳と異なる場合、詳細内容を基に修正
    if past_entries:
        past_entry = past_entries[0]  # 最初の一致するエントリを取得
        if past_entry['details'] != parsed_data.get('details', ''):
            # 仕訳を修正し、詳細内容を更新
            parsed_data['debit'] = past_entry['debit']
            parsed_data['credit'] = past_entry['credit']
            parsed_data['details'] = f"過去の仕訳に基づき修正: {past_entry['details']}"
    else:
        # 新規の仕訳の場合、MongoDBに保存
        save_journal_entry(parsed_data['debit'], parsed_data['credit'], parsed_data['amount'], parsed_data['details'])
    
    return parsed_data

def save_journal_entry(debit_account, credit_account, amount, details):
    entry = {
        "debit": debit_account,
        "credit": credit_account,
        "amount": amount,
        "details": details,
        "created_at": datetime.utcnow()  # 仕訳日時を追加
    }
    journal_entries_collection.insert_one(entry)  # MongoDBに挿入
    return entry

# 音声ファイルをWhisper APIでテキストに変換
@app.route('/api/voice_to_text', methods=['POST'])
def voice_to_text():
    if 'audio' not in request.files:
        return jsonify({'error': '音声ファイルが見つかりません'}), 400

    audio_file = request.files['audio']
    transcript = openai.Audio.transcribe("whisper-1", audio_file)
    
    # テキストを解析して仕訳データを生成
    text = transcript['text']
    parsed_data = analyze_text(text)

    # 仕訳データを過去のデータと比較
    corrected_data = compare_and_correct_journal_entries(parsed_data)

    return jsonify(corrected_data)

# QRコード生成
@app.route('/generate_qr', methods=['POST'])
def generate_qr():
    data = request.json

    # 必須情報の確認
    required_fields = ['sender', 'amount', 'debit', 'credit', 'recipients_count']
    for field in required_fields:
        if field not in data or not data[field] or data[field] in ['不明', '未設定', 0]:
            return jsonify({"error": f"{field}が不足しています"}), 400

    # MongoDBにトランザクションを保存（受信者は未定）
    transaction_id = str(uuid.uuid4())
    transaction_data = {
        "transaction_id": transaction_id,
        "sender": data['sender'],
        "amount": data['amount'],
        "debit": data['debit'],
        "credit": data['credit'],
        "details": data.get('details', ''),
        "status": "waiting_for_receivers",
        "recipients_count": data['recipients_count'],
        "received_count": 0,
        "recipients": [],  # 受信者のリスト
        "created_at": datetime.now(timezone.utc)
    }
    transactions_collection.insert_one(transaction_data)

    # QRコード生成
    qr_content = f"http://localhost:5000/receive_transaction/{transaction_id}"
    qr = qrcode.make(qr_content)
    qr_path = f"static/qr_codes/{transaction_id}.png"
    os.makedirs(os.path.dirname(qr_path), exist_ok=True)
    qr.save(qr_path)

    # QRコードの有効期限を設定（3分後に自動で期限切れ処理）
    threading.Thread(target=expire_transaction, args=(transaction_id, 3)).start()

    return jsonify({"qr_code_url": qr_path, "transaction_id": transaction_id})

# トランザクションの期限切れ処理
def expire_transaction(transaction_id, minutes):
    # 指定された時間だけ待機
    threading.Event().wait(minutes * 60)

    transaction = transactions_collection.find_one({"transaction_id": transaction_id})

    if transaction and transaction['status'] == 'waiting_for_receivers':
        # 期限切れになった場合の処理
        transactions_collection.update_one(
            {"transaction_id": transaction_id},
            {"$set": {"status": "expired"}}
        )
        print(f"Transaction {transaction_id} has expired.")

# QRコードを読み取った相手が受信者として特定される
@app.route('/receive_transaction/<transaction_id>', methods=['GET', 'POST'])
def receive_transaction(transaction_id):
    if request.method == 'POST':
        data = request.form
        receiver_municipality = data.get('receiver_municipality')
        receiver = data.get('receiver')

        if not receiver or not receiver_municipality:
            return jsonify({'message': 'receiver and receiver_municipality are required.'}), 400

        # トランザクションを取得
        transaction = transactions_collection.find_one({"transaction_id": transaction_id})

        if not transaction:
            return jsonify({"error": "トランザクションが見つかりません"}), 404

        if transaction['status'] != 'waiting_for_receivers':
            return jsonify({"message": "このトランザクションは既に処理済みか期限切れです。"}), 400

        # 既に受信した人数を確認
        if transaction['received_count'] >= transaction['recipients_count']:
            return jsonify({"message": "受信者の上限に達しました。"}), 400

        # 受信者情報を追加
        transactions_collection.update_one(
            {"transaction_id": transaction_id},
            {
                "$inc": {"received_count": 1},
                "$push": {"recipients": {
                    "receiver": receiver,
                    "receiver_municipality": receiver_municipality,
                    "received_at": datetime.now(timezone.utc)
                }}
            }
        )

        # トランザクションが全員に受信された場合、ステータスを更新
        transaction = transactions_collection.find_one({"transaction_id": transaction_id})
        if transaction['received_count'] == transaction['recipients_count']:
            transactions_collection.update_one(
                {"transaction_id": transaction_id},
                {"$set": {"status": "all_received"}}
            )

        return redirect(url_for('confirm_transaction', transaction_id=transaction_id))

    else:
        # 受信者情報入力フォームを表示
        return f'''
            <form method="post" action="">
                <label for="receiver">あなたのお名前を入力してください：</label>
                <input type="text" id="receiver" name="receiver"><br>
                <label for="receiver_municipality">あなたの市町村を入力してください（例：Asia-Tokyo）：</label>
                <input type="text" id="receiver_municipality" name="receiver_municipality"><br>
                <input type="hidden" name="transaction_id" value="{transaction_id}">
                <input type="submit" value="送信">
            </form>
        '''

# トランザクションの確認とMunicipal Chainへの送信
@app.route('/confirm_transaction/<transaction_id>', methods=['GET', 'POST'])
def confirm_transaction(transaction_id):
    transaction = transactions_collection.find_one({"transaction_id": transaction_id})

    if not transaction:
        return jsonify({"error": "トランザクションが見つかりません"}), 404

    if transaction['status'] == 'completed':
        return jsonify({"message": "このトランザクションは既に完了しています。"}), 400

    if request.method == 'POST':
        # Municipal Chainに送信
        for recipient in transaction.get('recipients', []):
            # 大陸情報を抽出
            receiver_municipality = recipient['receiver_municipality']
            receiver_continent = receiver_municipality.split('-')[0] if '-' in receiver_municipality else 'Default'

            municipal_chain_url = determine_municipal_chain(transaction.get('sender_municipality'), receiver_municipality)
            
            individual_transaction = {
                "transaction_id": transaction_id,
                "sender": transaction['sender'],
                "amount": transaction['amount'], # 各受信者に対して全額を設定
                "debit": transaction['debit'],
                "credit": transaction['credit'],
                "details": transaction['details'],
                "receiver": recipient['receiver'],
                "receiver_municipality": receiver_municipality,
                "status": "receive",
                "created_at": transaction['created_at'],
                "updated_at": datetime.now(timezone.utc)
            }

            response = requests.post(f'{municipal_chain_url}/receive_transaction', json=individual_transaction)
            if response.status_code not in [200, 202]:
                try:
                    error_msg = response.json().get('message', 'Failed to process transaction.')
                except json.JSONDecodeError:
                    error_msg = 'Failed to process transaction.'
                return jsonify({'message': error_msg}), response.status_code

        # トランザクションのステータスを 'completed' に更新
        transactions_collection.update_one(
            {"transaction_id": transaction_id},
            {"$set": {"status": "completed", "updated_at": datetime.now(timezone.utc)}}
        )

        return jsonify({"message": "トランザクションがMunicipal Chainに送信されました", "transaction": transaction})
    else:
        if transaction['status'] == 'all_received':
            # 送信者にトランザクションの詳細を表示して確認を促す
            recipients_info = ''.join([f"<li>{recipient['receiver']} ({recipient['receiver_municipality']})</li>" for recipient in transaction['recipients']])
            return f'''
                <h2>トランザクションの確認</h2>
                <p>送信者: {transaction['sender']}</p>
                <p>金額: {transaction['amount']}</p>
                <p>受信者数: {transaction['recipients_count']}</p>
                <p>受信者一覧:</p>
                <ul>{recipients_info}</ul>
                <form method="post">
                    <input type="submit" value="トランザクションを送信">
                </form>
            '''
        else:
            return jsonify({"message": "全ての受信者が受信していません。"}), 400

# Municipal Chain の URL を決定する関数
def determine_municipal_chain(sender_municipality, receiver_municipality):
    # municipalities.json を読み込む
    municipalities_file_path = os.path.join(BASE_DIR, 'municipalities.json')
    try:
        with open(municipalities_file_path, 'r', encoding='utf-8') as file:
            municipalities_data = json.load(file)
    except Exception as e:
        print(f"Error loading municipalities data: {e}")
        return None

    # Municipal Chain の URL を決定
    for continent, data in municipalities_data.items():
        for city in data.get('cities', []):
            city_id = f"{continent}-{city['name']}"
            if city_id == sender_municipality or city_id == receiver_municipality:
                city_port = city['city_flask_port']
                return f"http://127.0.0.1:{city_port}"

    # デフォルトの URL を返す
    default_city_port = municipalities_data['Default']['cities'][0]['city_flask_port']
    return f"http://127.0.0.1:{default_city_port}"

# Municipal Chainからの受信完了確認
@app.route('/complete_transaction/<transaction_id>', methods=['POST'])
def complete_transaction(transaction_id):
    transaction = transactions_collection.find_one({"transaction_id": transaction_id})

    if not transaction:
        return jsonify({"error": "トランザクションが見つかりません"}), 404

    # トランザクションを完了状態に更新
    transactions_collection.update_one(
        {"transaction_id": transaction_id},
        {"$set": {"status": "completed"}}
    )

    return jsonify({"message": "トランザクションが完了しました", "transaction": transaction})

if __name__ == '__main__':
    app.run(debug=True)
