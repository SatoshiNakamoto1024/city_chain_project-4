from flask import Flask, request, jsonify
import base64
import hashlib
from ntru import Ntru
from pqcrypto.sign.dilithium import generate_keypair, sign, verify
from datetime import datetime
import random

app = Flask(__name__)
ntru = Ntru()

# 鍵ペアの生成（例）
public_key = b'\x01' * 64  # 例として64バイトのダミー公開鍵
private_key = b'\x02' * 64  # 例として64バイトのダミー秘密鍵
class LatticeSigner:
    def __init__(self):
        self.public_key, self.private_key = generate_keypair()

    def sign(self, message):
        return sign(message, self.private_key)

    def verify(self, message, signature):
        return verify(message, signature, self.public_key)

# DPoSアルゴリズムの実装
class DPoS:
    def __init__(self, municipalities, private_key):
        self.municipalities = municipalities
        self.approved_representative = None
        self.private_key = private_key

    def elect_representative(self):
        self.approved_representative = random.choice(self.municipalities)
        return self.approved_representative

    def approve_transaction(self, transaction):
        if self.approved_representative:
            signature = sign(transaction['data'].encode(), self.private_key)
            transaction['signature'] = base64.b64encode(signature).decode()
            return True
        else:
            return False

# Proof of Placeの実装
class ProofOfPlace:
    def __init__(self, location, private_key):
        self.location = location
        self.timestamp = datetime.utcnow()
        self.private_key = private_key

    def generate_proof(self):
        proof_data = f"{self.location}{self.timestamp}".encode()
        return sign(proof_data, self.private_key)

    @staticmethod
    def verify_proof(proof, location, timestamp, public_key):
        proof_data = f"{location}{timestamp}".encode()
        return verify(proof_data, proof, public_key)

# Proof of Historyの実装
class ProofOfHistory:
    def __init__(self):
        self.sequence = []

    def add_event(self, event):
        self.sequence.append(event)

    def generate_hash(self):
        combined_events = ''.join(self.sequence)
        return hashlib.sha256(combined_events.encode()).hexdigest()

dpos = DPoS(['municipality1', 'municipality2', 'municipality3'])
poh = ProofOfHistory()

@app.route('/sign', methods=['POST'])
def sign():
    data = request.json.get('data')
    if data is None:
        return jsonify({'error': 'No data provided'}), 400

    signature = ntru.sign(data.encode('utf-8'), private_key)
    return jsonify({'signature': base64.b64encode(signature).decode('utf-8')})

@app.route('/verify', methods=['POST'])
def verify():
    data = request.json.get('data')
    signature = request.json.get('signature')
    if data is None or signature is None:
        return jsonify({'error': 'Data or signature not provided'}), 400

    signature_bytes = base64.b64decode(signature)
    is_valid = ntru.verify(data.encode('utf-8'), signature_bytes, public_key)
    return jsonify({'is_valid': is_valid})

@app.route('/elect_representative', methods=['POST'])
def elect_representative():
    representative = dpos.elect_representative()
    return jsonify({'representative': representative})

@app.route('/approve_transaction', methods=['POST'])
def approve_transaction():
    transaction = request.json
    if dpos.approve_transaction(transaction):
        return jsonify({'status': 'Transaction approved', 'transaction': transaction})
    else:
        return jsonify({'status': 'No representative elected'}), 400

@app.route('/generate_proof_of_place', methods=['POST'])
def generate_proof_of_place():
    location = request.json.get('location')
    if location is None:
        return jsonify({'error': 'No location provided'}), 400

    proof_of_place = ProofOfPlace(location)
    proof = proof_of_place.generate_proof()
    return jsonify({'proof': proof})

@app.route('/verify_proof_of_place', methods=['POST'])
def verify_proof_of_place():
    proof = request.json.get('proof')
    location = request.json.get('location')
    timestamp = request.json.get('timestamp')
    if proof is None or location is None or timestamp is None:
        return jsonify({'error': 'Missing proof, location, or timestamp'}), 400

    is_valid = ProofOfPlace.verify_proof(proof, location, timestamp)
    return jsonify({'is_valid': is_valid})

@app.route('/add_event_to_poh', methods=['POST'])
def add_event_to_poh():
    event = request.json.get('event')
    if event is None:
        return jsonify({'error': 'No event provided'}), 400

    poh.add_event(event)
    return jsonify({'status': 'Event added'})

@app.route('/generate_poh_hash', methods=['GET'])
def generate_poh_hash():
    poh_hash = poh.generate_hash()
    return jsonify({'poh_hash': poh_hash})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5001)
