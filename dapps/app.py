from flask import Flask, request, jsonify
import requests
import json
from datetime import datetime
from ntru import Ntru
from random import choice
import hashlib
import pytz

app = Flask(__name__)
ntru = Ntru()

class DPoS:
    def __init__(self, municipalities):
        self.municipalities = municipalities
        self.approved_representative = None

    def elect_representative(self):
        self.approved_representative = choice(self.municipalities)
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

dpos = DPoS(municipalities=["MunicipalityA", "MunicipalityB", "MunicipalityC"])

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

    if dpos.approve_transaction(transaction):
        response = requests.post('http://continental_main_chain:8001/transaction', json=transaction)
        return jsonify(response.json())
    else:
        return jsonify({"error": "Transaction not approved by DPoS"}), 403

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

@app.route('/generate_proof_of_place', methods=['POST'])
def generate_proof_of_place():
    data = request.json
    proof_of_place = ProofOfPlace(location=(data['latitude'], data['longitude']))
    proof = proof_of_place.generate_proof()
    return jsonify({"proof": proof, "timestamp": proof_of_place.timestamp.isoformat()})

@app.route('/verify_proof_of_place', methods=['POST'])
def verify_proof_of_place():
    data = request.json
    proof = data['proof']
    location = (data['latitude'], data['longitude'])
    timestamp = datetime.fromisoformat(data['timestamp'])
    is_valid = ProofOfPlace.verify_proof(proof, location, timestamp)
    return jsonify({"is_valid": is_valid})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
