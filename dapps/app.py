import sys
sys.path.append('/path/to/transaction')

from flask import Flask, request, jsonify
from transaction import create_transaction, recognize_speech, create_nft, sender_private_key, sender_public_key, receiver_public_key, LoveAction

app = Flask(__name__)

@app.route('/')
def home():
    return "Flask server is running"

@app.route('/create_transaction', methods=['POST'])
def create_transaction_endpoint():
    data = request.json
    transaction = create_transaction(data['sender'], data['receiver'], data['amount'])
    return jsonify(transaction), 201

@app.route('/recognize_and_create_nft', methods=['POST'])
def recognize_and_create_nft_endpoint():
    recognized_text = recognize_speech()
    if recognized_text:
        category = "本音で話す"
        dimension = "3次元：○○会社"
        content = "あなたはどの立場でその発言をしているんだ！もっと自分の立場を考えろ！"
        action = LoveAction(category, dimension, content, sender_public_key, receiver_public_key)
        create_nft(action)
        return jsonify({"message": "NFT created successfully"}), 201
    return jsonify({"error": "Speech recognition failed"}), 400

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)

