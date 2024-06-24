import hashlib
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

# Ntruクラスを使用するコード
if __name__ == "__main__":
    ntru = Ntru()
    public_key = b'public_key_1234567890abcdef'
    private_key = b'private_key_1234567890abcdef'

    message = "Hello, NTRU!"
    encrypted_message = ntru.encrypt(message.encode('utf-8'), public_key)
    decrypted_message = ntru.decrypt(encrypted_message, private_key)

    print(f"Encrypted Message: {encrypted_message}")
    print(f"Decrypted Message: {decrypted_message}")

    signature = ntru.sign(message.encode('utf-8'), private_key)
    is_valid = ntru.verify(message.encode('utf-8'), signature, public_key)

    print(f"Signature: {signature}")
    print(f"Is signature valid? {is_valid}")
