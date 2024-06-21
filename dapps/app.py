from flask import Flask, request, jsonify
import requests

app = Flask(__name__)

MUNICIPAL_CHAIN_HOST = "municipal_chain"

@app.route('/add_action', methods=['POST'])
def add_action():
    action_data = request.json
    # 市町村ブロックチェーンにデータを送信
    response = requests.post(f'http://{MUNICIPAL_CHAIN_HOST}:8081/add_block', json=action_data)
    return jsonify(response.json())

if __name__ == '__main__':
    app.run(port=5000, host='0.0.0.0')
