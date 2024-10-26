extern crate ntru; // 外部NTRUライブラリを使用するためのimport

use ntru::NtruEncrypt as NtruLib;
use ntru::keypair::KeyPair;
use ntru::params::NtruParams;
use std::error::Error;

pub struct NtruEncrypt {
    keypair: KeyPair,
}

impl NtruEncrypt {
    pub fn new() -> Self {
        // NTRUのパラメータを設定 (EES1087EP2はセキュリティレベルの一例)
        let params = NtruParams::default();
        let keypair = NtruLib::generate_keypair(&params).expect("Failed to generate NTRU keypair");
        NtruEncrypt { keypair }
    }

    // NTRUの公開鍵を取得
    pub fn get_public_key(&self) -> Vec<u8> {
        self.keypair.public_key.clone()
    }

    // NTRUの秘密鍵を取得
    pub fn get_private_key(&self) -> Vec<u8> {
        self.keypair.private_key.clone()
    }

    // NTRU暗号化関数
    pub fn encrypt(&self, message: &[u8], public_key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let params = NtruParams::default();
        let encrypted_message = NtruLib::encrypt(&params, message, public_key)?;
        Ok(encrypted_message)
    }

    // NTRU復号化関数
    pub fn decrypt(&self, encrypted_message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let params = NtruParams::default();
        let decrypted_message = NtruLib::decrypt(&params, encrypted_message, private_key)?;
        Ok(decrypted_message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ntru_encryption() {
        let ntru = NtruEncrypt::new();
        let message = b"Hello, NTRU!";
        
        // 公開鍵を使って暗号化
        let public_key = ntru.get_public_key();
        let encrypted_message = ntru.encrypt(message, &public_key).expect("Encryption failed");
        
        // 秘密鍵を使って復号化
        let private_key = ntru.get_private_key();
        let decrypted_message = ntru.decrypt(&encrypted_message, &private_key).expect("Decryption failed");
        
        assert_eq!(message.to_vec(), decrypted_message, "Decrypted message does not match original");
    }
}
