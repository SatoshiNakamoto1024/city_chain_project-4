extern crate ntru; // NTRUライブラリを使用するためのimport

use ntru::keypair::KeyPair;
use ntru::params::NtruParams;
use std::error::Error;

pub struct NtruParam {
    pub keypair: KeyPair,
}

impl NtruParam {
    pub fn new() -> Self {
        // NTRUのパラメータを設定 (ここではデフォルトのパラメータを使用)
        let params = NtruParams::default();
        let keypair = ntru::NtruEncrypt::generate_keypair(&params)
            .expect("Failed to generate NTRU keypair");
        NtruParam { keypair }
    }

    // 公開鍵を取得するメソッド
    pub fn get_public_key(&self) -> Vec<u8> {
        self.keypair.public_key.clone()
    }

    // 秘密鍵を取得するメソッド
    pub fn get_private_key(&self) -> Vec<u8> {
        self.keypair.private_key.clone()
    }

    // NTRUパラメータを生成する関数（鍵のペアを返す）
    pub fn generate_params(&self) -> (Vec<u8>, Vec<u8>) {
        let public_key = self.get_public_key();
        let private_key = self.get_private_key();
        (public_key, private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_params() {
        let ntru_param = NtruParam::new();
        let (public_key, private_key) = ntru_param.generate_params();

        // 公開鍵と秘密鍵の長さを確認
        assert_eq!(public_key.len(), 935); // 例：NTRU EES1087EP2 の場合の公開鍵長
        assert_eq!(private_key.len(), 1574); // 例：NTRU EES1087EP2 の場合の秘密鍵長
    }
}
