import base64
import json
from flask import Flask, request, jsonify
from ntru import Ntru

app = Flask(__name__)

# 鍵ペアの生成（例）
ntru = Ntru()
public_key, private_key = ntru.generate_keypair()

def load_public_key():
    # 公開鍵をロードする実装
    return public_key

def load_private_key():
    # 秘密鍵をロードする実装
    return private_key

@app.route('/sign', methods=['POST'])
def sign_data():
    data = request.json.get('data')
    private_key = load_private_key()
    data_bytes = data.encode('utf-8')
    
    ntru = Ntru()
    signature = ntru.sign(data_bytes, private_key)
    signature_base64 = base64.b64encode(signature).decode('utf-8')
    
    return jsonify({'signature': signature_base64})

@app.route('/verify', methods=['POST'])
def verify_signature():
    data = request.json.get('data')
    signature = request.json.get('signature')
    public_key = load_public_key()
    signature_bytes = base64.b64decode(signature)
    data_bytes = data.encode('utf-8')
    
    ntru = Ntru()
    is_valid = ntru.verify(data_bytes, signature_bytes, public_key)
    
    return jsonify({'valid': is_valid})

if __name__ == "__main__":
    app.run(host='0.0.0.0', port=5001)
