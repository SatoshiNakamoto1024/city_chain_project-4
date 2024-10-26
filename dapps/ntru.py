import hashlib
import base64
from Cryptodome.PublicKey import RSA
from Cryptodome.Signature import pkcs1_15
from Cryptodome.Hash import SHA256
from pqcrypto.ntru import encrypt as ntru_encrypt, decrypt as ntru_decrypt, generate_keypair  # NTRU暗号ライブラリを使用

class Ntru:
    def __init__(self):
        # NTRUの公開鍵と秘密鍵を生成
        self.public_key, self.private_key = generate_keypair()

    def encrypt(self, plaintext):
        # NTRUによる暗号化
        encrypted = ntru_encrypt(plaintext, self.public_key)
        return encrypted

    def decrypt(self, ciphertext):
        # NTRUによる復号化
        decrypted = ntru_decrypt(ciphertext, self.private_key)
        return decrypted

    def sign(self, message, private_key):
        # 署名の実装例 (RSA署名)
        hashed_message = SHA256.new(message)
        signature = pkcs1_15.new(private_key).sign(hashed_message)
        return signature

    def verify(self, message, signature, public_key):
        # 署名検証の実装例 (RSA署名検証)
        hashed_message = SHA256.new(message)
        try:
            pkcs1_15.new(public_key).verify(hashed_message, signature)
            return True
        except (ValueError, TypeError):
            return False

# Ntruクラスを使用するコード
if __name__ == "__main__":
    ntru = Ntru()

    message = b"Hello, NTRU!"
    
    # NTRU暗号を使ってメッセージを暗号化・復号化
    encrypted_message = ntru.encrypt(message)
    decrypted_message = ntru.decrypt(encrypted_message)

    print(f"Encrypted Message: {encrypted_message}")
    print(f"Decrypted Message: {decrypted_message.decode('utf-8')}")

    # RSA署名と検証
    rsa_key = RSA.generate(2048)
    private_key = rsa_key
    public_key = rsa_key.publickey()

    signature = ntru.sign(message, private_key)
    is_valid = ntru.verify(message, signature, public_key)

    print(f"Signature: {base64.b64encode(signature).decode('utf-8')}")
    print(f"Is signature valid? {is_valid}")
