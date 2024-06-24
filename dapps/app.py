from flask import Flask, request, jsonify
import requests
import json
from ntru import Ntru

app = Flask(__name__)
ntru = Ntru()

@app.route('/create_transaction', methods=['POST'])
def create_transaction():
    data = request.json
    transaction = {
        "sender": data['sender'],
        "receiver": data['receiver'],
        "amount": data['amount'],
        "timestamp": data['timestamp'],
        "transaction_id": data['transaction_id'],
        "verifiable_credential": data['verifiable_credential'],
        "signature": ntru.sign(json.dumps(data).encode(), data['private_key'])
    }
    response = requests.post('http://continental_main_chain:8001/transaction', json=transaction)
    return jsonify(response.json())

@app.route('/add_block', methods=['POST'])
def add_block():
    data = request.json
    response = requests.post('http://continental_main_chain:8001/add_block', json=data)
    return jsonify(response.json())

@app.route('/sign_transaction', methods=['POST'])
def sign_transaction():
    data = request.json
    transaction = json.dumps(data)
    signature = ntru.sign(transaction.encode(), data['private_key'])
    return jsonify({"signature": signature.hex()})

@app.route('/verify_signature', methods=['POST'])
def verify_signature():
    data = request.json
    transaction = json.dumps(data['transaction'])
    signature = bytes.fromhex(data['signature'])
    public_key = data['public_key']
    is_valid = ntru.verify(transaction.encode(), signature, public_key)
    return jsonify({"is_valid": is_valid})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
