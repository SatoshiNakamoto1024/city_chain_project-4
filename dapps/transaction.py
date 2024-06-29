from datetime import datetime
import hashlib
import json
import qrcode
import speech_recognition as sr
from Crypto.PublicKey import RSA
from Crypto.Signature import pkcs1_15
from Crypto.Hash import SHA256
from Crypto.Cipher import PKCS1_OAEP 

class Ntru:
    def __init__(self):
        pass

    def encrypt(self, plaintext, public_key):
        # 公開鍵を使用して暗号化
        key = RSA.import_key(public_key)
        cipher_rsa = PKCS1_OAEP.new(key)
        return cipher_rsa.encrypt(plaintext)

    def decrypt(self, ciphertext, private_key):
        # 秘密鍵を使用して復号化
        key = RSA.import_key(private_key)
        cipher_rsa = PKCS1_OAEP.new(key)
        return cipher_rsa.decrypt(ciphertext)

    def sign(self, message, private_key):
        # 秘密鍵を使用してメッセージに署名
        key = RSA.import_key(private_key)
        h = SHA256.new(message)
        return pkcs1_15.new(key).sign(h)

    def verify(self, message, signature, public_key):
        # 公開鍵を使用して署名を検証
        key = RSA.import_key(public_key)
        h = SHA256.new(message)
        try:
            pkcs1_15.new(key).verify(h, signature)
            return True
        except (ValueError, TypeError):
            return False

# Ntruクラスの使用
ntru = Ntru()

# トランザクション作成関数
def create_transaction(sender, receiver, amount, private_key):
    transaction = {
        "sender": sender,
        "receiver": receiver,
        "amount": amount,
        "timestamp": datetime.utcnow().isoformat(),
        "transaction_id": hashlib.sha256(f"{sender}{receiver}{amount}{datetime.utcnow().isoformat()}".encode()).hexdigest()
    }
    transaction["signature"] = ntru.sign(json.dumps(transaction).encode(), private_key).hex()
    return transaction

# 音声認識関数
def recognize_speech():
    recognizer = sr.Recognizer()
    with sr.Microphone() as source:
        print("Say something:")
        audio = recognizer.listen(source)
    try:
        text = recognizer.recognize_google(audio, language="ja-JP")
        print(f"You said: {text}")
        return text
    except sr.UnknownValueError:
        print("Google Speech Recognition could not understand audio")
        return None
    except sr.RequestError as e:
        print(f"Could not request results from Google Speech Recognition service; {e}")
        return None

# LoveAction クラス
class LoveAction:
    def __init__(self, category, dimension, content, sender, receiver):
        self.category = category
        self.dimension = dimension
        self.content = content
        self.sender = sender
        self.receiver = receiver
        self.timestamp = datetime.utcnow().isoformat()
        self.transaction_id = self.generate_transaction_id()
        self.signature = None

    def generate_transaction_id(self):
        data = f"{self.sender}{self.receiver}{self.timestamp}{self.category}{self.dimension}{self.content}"
        return hashlib.sha256(data.encode()).hexdigest()

    def to_dict(self):
        return {
            "category": self.category,
            "dimension": self.dimension,
            "content": self.content,
            "sender": self.sender,
            "receiver": self.receiver,
            "timestamp": self.timestamp,
            "transaction_id": self.transaction_id,
            "signature": self.signature
        }

    def sign_transaction(self, private_key):
        self.signature = ntru.sign(json.dumps(self.to_dict(), sort_keys=True).encode(), private_key).hex()

    def verify_signature(self, public_key):
        return ntru.verify(json.dumps(self.to_dict(), sort_keys=True).encode(), bytes.fromhex(self.signature), public_key)

# NFT作成関数
def create_nft(action, private_key):
    action.sign_transaction(private_key)
    nft_data = json.dumps(action.to_dict())
    qr = qrcode.QRCode(version=1, box_size=10, border=5)
    qr.add_data(nft_data)
    qr.make(fit=True)
    img = qr.make_image(fill='black', back_color='white')
    img.save("love_action_qr.png")
    print("QR code generated and saved as 'love_action_qr.png'")

# 鍵ペアの生成（例）
key = RSA.generate(2048)
sender_private_key = key.export_key()
sender_public_key = key.publickey().export_key()
receiver_key = RSA.generate(2048)
receiver_private_key = receiver_key.export_key()
receiver_public_key = receiver_key.publickey().export_key()
