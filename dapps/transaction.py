from datetime import datetime
import hashlib
import json
import qrcode
from ecdsa import SigningKey, VerifyingKey, NIST384p
import speech_recognition as sr

# トランザクション作成関数
def create_transaction(sender, receiver, amount):
    transaction = {
        "sender": sender,
        "receiver": receiver,
        "amount": amount,
        "timestamp": datetime.utcnow().isoformat(),
        "transaction_id": hashlib.sha256(f"{sender}{receiver}{amount}{datetime.utcnow().isoformat()}".encode()).hexdigest()
    }
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
        sk = SigningKey.from_string(bytes.fromhex(private_key), curve=NIST384p)
        self.signature = sk.sign(json.dumps(self.to_dict(), sort_keys=True).encode()).hex()

    def verify_signature(self, public_key):
        vk = VerifyingKey.from_string(bytes.fromhex(public_key), curve=NIST384p)
        try:
            return vk.verify(bytes.fromhex(self.signature), json.dumps(self.to_dict(), sort_keys=True).encode())
        except:
            return False

# NFT作成関数
def create_nft(action):
    action.sign_transaction(sender_private_key)
    nft_data = json.dumps(action.to_dict())
    qr = qrcode.QRCode(version=1, box_size=10, border=5)
    qr.add_data(nft_data)
    qr.make(fit=True)
    img = qr.make_image(fill='black', back_color='white')
    img.save("love_action_qr.png")
    print("QR code generated and saved as 'love_action_qr.png'")

# 鍵ペアの生成（例）
sender_private_key = SigningKey.generate(curve=NIST384p).to_string().hex()
sender_public_key = SigningKey.from_string(bytes.fromhex(sender_private_key), curve=NIST384p).verifying_key.to_string().hex()
receiver_private_key = SigningKey.generate(curve=NIST384p).to_string().hex()
receiver_public_key = SigningKey.from_string(bytes.fromhex(receiver_private_key), curve=NIST384p).verifying_key.to_string().hex()
