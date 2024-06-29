import hashlib
import random
from datetime import datetime 
import base64
from Crypto.PublicKey import RSA
from Crypto.Signature import pkcs1_15
from Crypto.Hash import SHA256

class Ntru:
    def __init__(self):
        pass

    def encrypt(self, plaintext, public_key):
        # 暗号化の簡単な実装例
        return bytes([p ^ k for p, k in zip(plaintext, public_key)])

    def decrypt(self, ciphertext, private_key):
        # 復号化の簡単な実装例
        return bytes([c ^ k for c, k in zip(ciphertext, private_key)])

    def sign(self, message, private_key):
        # 署名の簡単な実装例
        hashed_message = SHA256.new(message)
        signature = pkcs1_15.new(private_key).sign(hashed_message)
        return signature

    def verify(self, message, signature, public_key):
        # 署名検証の簡単な実装例
        hashed_message = SHA256.new(message)
        try:
            pkcs1_15.new(public_key).verify(hashed_message, signature)
            return True
        except (ValueError, TypeError):
            return False

# DPoSの実装
class DPoS:
    def __init__(self, municipalities):
        self.municipalities = municipalities
        self.approved_representative = None

    def elect_representative(self):
        self.approved_representative = random.choice(self.municipalities)
        return self.approved_representative

    def approve_transaction(self, transaction):
        if self.approved_representative:
            transaction['signature'] = f"approved_by_{self.approved_representative}"
            return True
        else:
            return False

# Proof of Placeの実装
    def generate_proof(self):
        proof_string = f"{self.location}{self.timestamp}"
        return hashlib.sha256(proof_string.encode()).hexdigest()

    @staticmethod
    def verify_proof(proof, location, timestamp):
        proof_string = f"{location}{timestamp}"
        computed_proof = hashlib.sha256(proof_string.encode()).hexdigest()
        return computed_proof == proof

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
