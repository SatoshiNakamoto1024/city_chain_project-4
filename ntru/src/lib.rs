extern crate rand;
extern crate serde;
extern crate serde_json;

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct NTRUKeys {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

pub fn generate_ntru_keys() -> NTRUKeys {
    let mut rng = rand::thread_rng();
    let public_key = (0..64).map(|_| rng.gen::<u8>()).collect();
    let private_key = (0..64).map(|_| rng.gen::<u8>()).collect();
    NTRUKeys { public_key, private_key }
}

pub fn ntru_encrypt(data: &[u8], public_key: &[u8]) -> Vec<u8> {
    data.iter().zip(public_key).map(|(&d, &k)| d ^ k).collect()
}

pub fn ntru_decrypt(encrypted_data: &[u8], private_key: &[u8]) -> Vec<u8> {
    encrypted_data.iter().zip(private_key).map(|(&e, &k)| e ^ k).collect()
}

pub fn sign_transaction(data: &[u8], private_key: &[u8]) -> Vec<u8> {
    private_key.iter().zip(data).map(|(&k, &d)| k ^ d).collect()
}

pub fn verify_signature(data: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    let calculated_signature: Vec<u8> = public_key.iter().zip(data).map(|(&k, &d)| k ^ d).collect();
    calculated_signature == signature
}

fn main() {
    // NTRU鍵ペアの生成
    let keys = generate_ntru_keys();
    println!("Public Key: {:?}", keys.public_key);
    println!("Private Key: {:?}", keys.private_key);

    // データの暗号化と復号
    let data = b"Hello, NTRU!";
    let encrypted_data = ntru_encrypt(data, &keys.public_key);
    let decrypted_data = ntru_decrypt(&encrypted_data, &keys.private_key);
    println!("Encrypted Data: {:?}", encrypted_data);
    println!("Decrypted Data: {:?}", String::from_utf8(decrypted_data).unwrap());

    // トランザクションの署名と検証
    let signature = sign_transaction(data, &keys.private_key);
    let is_valid = verify_signature(data, &signature, &keys.public_key);
    println!("Signature: {:?}", signature);
    println!("Is signature valid? {}", is_valid);
}
