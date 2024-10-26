from datetime import datetime
import hashlib
import json
import qrcode  # モジュール全体をインポート
from typing import Optional  # 型ヒントのために追加
from Cryptodome.PublicKey import RSA
from Cryptodome.Signature import pkcs1_15
from Cryptodome.Hash import SHA256
from Cryptodome.Cipher import PKCS1_OAEP
from pqcrypto.ntru import encrypt as ntru_encrypt, decrypt as ntru_decrypt, generate_keypair  # NTRU暗号を使用

# 暗号化・署名クラス
class CryptoUtils:
    def __init__(self):
        # NTRUの公開鍵と秘密鍵を生成
        self.ntru_public_key, self.ntru_private_key = generate_keypair()

    def encrypt(self, plaintext: str, public_key: bytes) -> bytes:
        # NTRUによる暗号化
        encrypted = ntru_encrypt(plaintext.encode(), public_key)
        return encrypted

    def decrypt(self, ciphertext: bytes, private_key: bytes) -> str:
        # NTRUによる復号化
        decrypted = ntru_decrypt(ciphertext, private_key)
        return decrypted.decode()

    def rsa_encrypt(self, plaintext: str, public_key: bytes) -> bytes:
        key = RSA.import_key(public_key)
        cipher_rsa = PKCS1_OAEP.new(key)
        return cipher_rsa.encrypt(plaintext.encode())

    def rsa_decrypt(self, ciphertext: bytes, private_key: bytes) -> str:
        key = RSA.import_key(private_key)
        cipher_rsa = PKCS1_OAEP.new(key)
        return cipher_rsa.decrypt(ciphertext).decode()

    def sign(self, message: bytes, private_key: bytes) -> bytes:
        key = RSA.import_key(private_key)
        h = SHA256.new(message)
        return pkcs1_15.new(key).sign(h)

    def verify(self, message: bytes, signature: bytes, public_key: bytes) -> bool:
        key = RSA.import_key(public_key)
        h = SHA256.new(message)
        try:
            pkcs1_15.new(key).verify(h, signature)
            return True
        except (ValueError, TypeError):
            return False

# CryptoUtils クラスのインスタンス化
crypto_utils = CryptoUtils()

# LoveAction クラス
class LoveAction:
    def __init__(
        self,
        category: str,
        dimension: str,
        content: str,
        sender: str,
        receiver: str,
        sender_municipality: str,
        receiver_municipality: str,
    ):
        self.category = category
        self.dimension = dimension
        self.content = content
        self.sender = sender
        self.receiver = receiver
        self.sender_municipality = sender_municipality
        self.receiver_municipality = receiver_municipality
        self.timestamp = datetime.utcnow().isoformat()
        self.transaction_id = self.generate_transaction_id()
        self.signature: Optional[str] = None  # 型アノテーションを追加

    def generate_transaction_id(self) -> str:
        data = f"{self.sender}{self.receiver}{self.timestamp}{self.category}{self.dimension}{self.content}"
        return hashlib.sha256(data.encode()).hexdigest()

    def to_dict(self) -> dict:
        return {
            "category": self.category,
            "dimension": self.dimension,
            "content": self.content,
            "sender": self.sender,
            "receiver": self.receiver,
            "sender_municipality": self.sender_municipality,
            "receiver_municipality": self.receiver_municipality,
            "timestamp": self.timestamp,
            "transaction_id": self.transaction_id,
            "signature": self.signature,
            # 他の必要なフィールドを追加
        }

    def sign_transaction(self, private_key: bytes):
        transaction_data = self.to_dict()
        transaction_data.pop('signature', None)
        self.signature = crypto_utils.sign(
            json.dumps(transaction_data, sort_keys=True).encode(),
            private_key
        ).hex()

    def verify_signature(self, public_key: bytes) -> bool:
        if self.signature is None:
            print("No signature to verify.")
            return False
        transaction_data = self.to_dict()
        transaction_data.pop('signature', None)
        try:
            signature_bytes = bytes.fromhex(self.signature)
        except ValueError as e:
            print(f"Invalid signature format: {e}")
            return False
        return crypto_utils.verify(
            json.dumps(transaction_data, sort_keys=True).encode(),
            signature_bytes,
            public_key
        )

# トランザクション作成関数
def create_transaction(
    sender: str,
    receiver: str,
    amount: float,
    private_key: bytes,
    sender_municipality: str,
    receiver_municipality: str
) -> dict:
    timestamp = datetime.utcnow().isoformat()
    transaction = {
        "sender": sender,
        "receiver": receiver,
        "amount": amount,
        "timestamp": timestamp,
        "sender_municipality": sender_municipality,
        "receiver_municipality": receiver_municipality,
        # 他の必要なフィールドを追加
    }
    # トランザクションIDの生成
    transaction_id_data = f"{sender}{receiver}{amount}{timestamp}"
    transaction["transaction_id"] = hashlib.sha256(transaction_id_data.encode()).hexdigest()

    # 署名対象のデータを準備（署名フィールドを含めない）
    transaction_for_signing = transaction.copy()
    transaction_for_signing.pop('signature', None)

    # トランザクションの署名
    signature = crypto_utils.sign(
        json.dumps(transaction_for_signing, sort_keys=True).encode(),
        private_key
    )
    transaction["signature"] = signature.hex()
    return transaction

# NFT作成関数
def create_nft(action: LoveAction, private_key: bytes, output_filename="love_action_qr.png"):
    action.sign_transaction(private_key)
    nft_data = json.dumps(action.to_dict())
    qr = qrcode.QRCode(version=1, box_size=10, border=5)  # 修正ポイント
    qr.add_data(nft_data)
    qr.make(fit=True)
    img = qr.make_image(fill_color='black', back_color='white')  # 修正ポイント
    img.save(output_filename)
    print(f"QR code generated and saved as '{output_filename}'")

# 鍵ペアの生成または読み込み関数
def load_or_generate_keys(private_key_file: str, public_key_file: str, key_size=2048):
    try:
        # 鍵ペアが既に存在する場合は読み込み（例としてファイルから）
        with open(private_key_file, 'rb') as f:
            private_key = f.read()
        with open(public_key_file, 'rb') as f:
            public_key = f.read()
    except FileNotFoundError:
        # 鍵ペアが存在しない場合は新規生成
        key = RSA.generate(key_size)
        private_key = key.export_key()
        public_key = key.publickey().export_key()
        # 鍵をファイルに保存
        with open(private_key_file, 'wb') as f:
            f.write(private_key)
        with open(public_key_file, 'wb') as f:
            f.write(public_key)

    return private_key, public_key

# メイン処理
if __name__ == "__main__":
    # 鍵ペアのロードまたは生成
    sender_private_key, sender_public_key = load_or_generate_keys('sender_private_key.pem', 'sender_public_key.pem')
    receiver_private_key, receiver_public_key = load_or_generate_keys('receiver_private_key.pem', 'receiver_public_key.pem')

    # トランザクションの作成
    sender = "Alice"
    receiver = "Bob"
    amount = 100.0
    sender_municipality = "Asia-Tokyo"
    receiver_municipality = "Europe-London"

    transaction = create_transaction(
        sender,
        receiver,
        amount,
        sender_private_key,
        sender_municipality,
        receiver_municipality
    )
    print("Transaction created:", transaction)

    # トランザクションの署名検証
    transaction_for_verification = transaction.copy()
    signature_hex = transaction_for_verification.pop('signature', None)
    if signature_hex is not None:
        try:
            signature_bytes = bytes.fromhex(signature_hex)
            is_valid = crypto_utils.verify(
                json.dumps(transaction_for_verification, sort_keys=True).encode(),
                signature_bytes,
                sender_public_key
            )
            print("Signature valid:", is_valid)
        except ValueError as e:
            print(f"Error in signature verification: {e}")
    else:
        print("No signature found in the transaction.")

    # LoveAction の作成とNFT生成
    action = LoveAction(
        category="Affection",
        dimension="Emotional",
        content="Sending love and gratitude",
        sender=sender,
        receiver=receiver,
        sender_municipality=sender_municipality,
        receiver_municipality=receiver_municipality
    )

    create_nft(action, sender_private_key)

    # LoveAction の署名検証
    is_action_valid = action.verify_signature(sender_public_key)
    print("LoveAction signature valid:", is_action_valid)
