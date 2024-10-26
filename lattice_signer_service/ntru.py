import hashlib
import random
from datetime import datetime 
import base64
from Crypto.PublicKey import RSA
from Crypto.Signature import pkcs1_15
from Crypto.Hash import SHA256
from pqcrypto.sign.dilithium import generate_keypair, sign, verify
import hashlib

class Ntru:
    def __init__(self):
        self.public_key, self.private_key = generate_keypair()  # 鍵ペア生成

    def encrypt(self, plaintext, public_key):
        # NTRUではなく、署名の部分に注力するために削除
        pass

    def decrypt(self, ciphertext, private_key):
        # NTRUではなく、署名の部分に注力するために削除
        pass

    def sign(self, message):
        # メッセージに対する署名
        return sign(message, self.private_key)

    def verify(self, message, signature):
        # 署名の検証
        return verify(message, signature, self.public_key)

# Ntruクラスを使用するコード
if __name__ == "__main__":
    ntru = Ntru()
    message = "Hello, NTRU!"
    signature = ntru.sign(message.encode('utf-8'))
    is_valid = ntru.verify(message.encode('utf-8'), signature)

    print(f"Signature: {signature}")
    print(f"Is signature valid? {is_valid}")

# DPoSの実装
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

# 鍵ペアの生成（例）
key = RSA.generate(2048)
sender_private_key = key
sender_public_key = key.publickey()
receiver_key = RSA.generate(2048)
receiver_private_key = receiver_key
receiver_public_key = receiver_key.publickey()

# Ntruクラスを使用するコード
if __name__ == "__main__":
    ntru = Ntru()
    public_key = sender_public_key.export_key()
    private_key = sender_private_key.export_key()

    message = "Hello, NTRU!"
    encrypted_message = ntru.encrypt(message.encode('utf-8'), public_key)
    decrypted_message = ntru.decrypt(encrypted_message, private_key)

    print(f"Encrypted Message: {encrypted_message}")
    print(f"Decrypted Message: {decrypted_message}")

    signature = ntru.sign(message.encode('utf-8'), private_key)
    is_valid = ntru.verify(message.encode('utf-8'), signature, public_key)

    print(f"Signature: {signature}")
    print(f"Is signature valid? {is_valid}")
