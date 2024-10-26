extern crate ntru; // NTRUライブラリを使用
use ntru::keypair::KeyPair;
use ntru::sign::{NtruSign as NtruSignLib, NtruSignKeyPair, NtruSignature};
use ntru::params::NtruParams;
use sha2::{Sha256, Digest};
use std::error::Error;

pub struct NtruSign {
    pub sign_keypair: NtruSignKeyPair, // 署名用の鍵ペアを保持
}

impl NtruSign {
    pub fn new() -> Self {
        // NTRUの署名アルゴリズムに必要なパラメータを初期化
        let params = NtruParams::default();
        let sign_keypair = NtruSignLib::generate_keypair(&params)
            .expect("Failed to generate NTRU signing keypair");
        NtruSign { sign_keypair }
    }

    // メッセージに署名する関数
    pub fn sign(&self, message: &[u8]) -> NtruSignature {
        // ハッシュ化されたメッセージをNTRU署名アルゴリズムに基づいて署名
        let mut hasher = Sha256::new();
        hasher.update(message);
        let message_hash = hasher.finalize();
        
        NtruSignLib::sign(&message_hash, &self.sign_keypair)
            .expect("Failed to sign the message")
    }

    // 署名の検証を行う関数
    pub fn verify(&self, message: &[u8], signature: &NtruSignature) -> bool {
        // メッセージを再度ハッシュ化し、署名の検証を行う
        let mut hasher = Sha256::new();
        hasher.update(message);
        let message_hash = hasher.finalize();
        
        NtruSignLib::verify(&message_hash, signature, &self.sign_keypair.public_key)
            .unwrap_or(false) // 失敗した場合は検証失敗として扱う
    }

    // 公開鍵を取得するメソッド
    pub fn get_public_key(&self) -> Vec<u8> {
        self.sign_keypair.public_key.clone()
    }

    // 秘密鍵を取得するメソッド
    pub fn get_private_key(&self) -> Vec<u8> {
        self.sign_keypair.private_key.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let ntru_sign = NtruSign::new();
        let message = b"Hello, NTRU Signature!";
        
        // メッセージに署名
        let signature = ntru_sign.sign(message);

        // 署名を検証
        assert!(ntru_sign.verify(message, &signature));
    }

    #[test]
    fn test_invalid_signature() {
        let ntru_sign = NtruSign::new();
        let message = b"Hello, NTRU Signature!";
        let invalid_message = b"Hello, Invalid Signature!";
        
        // 正しいメッセージに署名
        let signature = ntru_sign.sign(message);

        // 間違ったメッセージで検証して失敗を確認
        assert!(!ntru_sign.verify(invalid_message, &signature));
    }
}
