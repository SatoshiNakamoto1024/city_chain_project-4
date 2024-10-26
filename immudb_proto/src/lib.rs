extern crate ntru; // NTRUクレートを使う

use ntru::NtruEncrypt; // NTRUの署名・暗号化ライブラリをインポート

// NTRUの公開鍵・秘密鍵を生成する関数
pub fn generate_keys() -> (Vec<u8>, Vec<u8>) {
    let ntru = NtruEncrypt::default();
    let (public_key, private_key) = ntru.generate_keypair();
    (public_key, private_key)
}

// データに署名を行う関数
pub fn sign_data(private_key: &[u8], data: &[u8]) -> Vec<u8> {
    let ntru = NtruEncrypt::default();
    ntru.sign(private_key, data).expect("Failed to sign data")
}

// 署名を検証する関数
pub fn verify_signature(public_key: &[u8], data: &[u8], signature: &[u8]) -> bool {
    let ntru = NtruEncrypt::default();
    ntru.verify(public_key, data, signature).unwrap_or(false)
}

pub fn immudb_proto_function() {
    println!("This is a function in immudb_proto crate with NTRU signing!");

    // 例として、キーの生成と署名・検証
    let (public_key, private_key) = generate_keys();
    let data = b"Sample data for signing";
    let signature = sign_data(&private_key, data);

    if verify_signature(&public_key, data, &signature) {
        println!("Signature verified successfully!");
    } else {
        println!("Signature verification failed!");
    }
}
