extern crate ntru;

use ntru::{generate_ntru_keys, ntru_encrypt, ntru_decrypt, sign_transaction, verify_signature, ProofOfPlace, ProofOfHistory};
use chrono::Utc;

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

    // Proof of Place の生成と検証
    let location = (37.7749, -122.4194); // サンプルの緯度経度（サンフランシスコ）
    let proof_of_place = ProofOfPlace::new(location);
    let proof = proof_of_place.generate_proof();
    let is_place_valid = ProofOfPlace::verify_proof(&proof, location, proof_of_place.timestamp);
    println!("Proof of Place: {:?}", proof);
    println!("Is Proof of Place valid? {}", is_place_valid);

    // Proof of History の生成
    let mut proof_of_history = ProofOfHistory::new();
    proof_of_history.add_event("Event 1");
    proof_of_history.add_event("Event 2");
    proof_of_history.add_event("Event 3");
    let history_hash = proof_of_history.generate_hash();
    println!("Proof of History Hash: {:?}", history_hash);

    // 現在の時間を表示
    let current_time = Utc::now();
    println!("Current time: {:?}", current_time);
}
