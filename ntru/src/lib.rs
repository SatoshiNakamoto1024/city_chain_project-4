use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use rand::Rng;
use rand::prelude::SliceRandom;
use ntru::NtruEncrypt; // NTRU暗号化ライブラリを使用

pub mod ntru_encrypt;
pub mod ntru_param;
pub mod ntru_sign;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NTRUKeys {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

// NTRUキーの生成
pub fn generate_ntru_keys() -> NTRUKeys {
    let (public_key, private_key) = NtruEncrypt::generate_keypair(); // NTRUライブラリを使用
    NTRUKeys {
        public_key: public_key.to_vec(),
        private_key: private_key.to_vec(),
    }
}

// NTRU暗号化
pub fn ntru_encrypt(data: &[u8], public_key: &[u8]) -> Vec<u8> {
    let mut ntru = NtruEncrypt::new();
    ntru.encrypt(data, public_key).expect("Encryption failed")
}

// NTRU復号化
pub fn ntru_decrypt(encrypted_data: &[u8], private_key: &[u8]) -> Vec<u8> {
    let mut ntru = NtruEncrypt::new();
    ntru.decrypt(encrypted_data, private_key).expect("Decryption failed")
}

// NTRU署名の生成
pub fn ntru_sign(data: &[u8], private_key: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hashed_data = hasher.finalize();
    let mut ntru = NtruEncrypt::new();
    ntru.sign(&hashed_data, private_key).expect("Signing failed")
}

// 署名の検証
pub fn verify_signature(data: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hashed_data = hasher.finalize();
    let mut ntru = NtruEncrypt::new();
    ntru.verify(&hashed_data, signature, public_key).unwrap_or(false)
}

// Proof of Placeの実装
#[derive(Serialize, Deserialize)]
pub struct ProofOfPlace {
    pub location: (f64, f64),
    pub timestamp: DateTime<Utc>,
}

impl ProofOfPlace {
    pub fn new(location: (f64, f64)) -> Self {
        ProofOfPlace {
            location,
            timestamp: Utc::now(),
        }
    }

    pub fn generate_proof(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}{:?}", self.location, self.timestamp).as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn verify_proof(proof: &str, location: (f64, f64), timestamp: DateTime<Utc>) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}{:?}", location, timestamp).as_bytes());
        let computed_proof = hex::encode(hasher.finalize());
        computed_proof == proof
    }
}

// Proof of Historyの実装
pub struct ProofOfHistory {
    pub sequence: Vec<String>,
}

impl ProofOfHistory {
    pub fn new() -> Self {
        ProofOfHistory {
            sequence: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: &str) {
        self.sequence.push(event.to_string());
    }

    pub fn generate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for event in &self.sequence {
            hasher.update(event.as_bytes());
        }
        hex::encode(hasher.finalize())
    }
}

// DPoSの実装
pub struct DPoS {
    pub municipalities: Vec<String>,
    pub approved_representative: Option<String>,
}

impl DPoS {
    pub fn new(municipalities: Vec<String>) -> Self {
        Self {
            municipalities,
            approved_representative: None,
        }
    }

    pub fn elect_representative(&mut self) -> String {
        let representative = self.municipalities.choose(&mut rand::thread_rng()).unwrap().clone();
        self.approved_representative = Some(representative.clone());
        representative
    }

    pub fn approve_transaction(&self, transaction: &mut Transaction) -> Result<&str, &str> {
        if let Some(representative) = &self.approved_representative {
            transaction.signature = format!("approved_by_{}", representative).as_bytes().to_vec();
            Ok("Transaction approved")
        } else {
            Err("No representative elected")
        }
    }
}

// トランザクション構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: String,
    pub municipality: String,
    pub timestamp: DateTime<Utc>,
    pub location: (f64, f64),
    pub love_action_level: i32,
    pub amount: f64,
    pub action_content: String,
    pub is_local: bool,
    pub close_flag: bool,
    pub approval_target: Option<String>,
    pub sender_public_key: String,
    pub receiver_public_key: String,
    pub signature: Vec<u8>,
    pub location_hash: Vec<u8>,
    pub received_timestamp: Option<DateTime<Utc>>,
    pub recipient_location: Option<(f64, f64)>,
    pub fee: f64,
}

impl Transaction {
    pub fn new(
        transaction_id: String,
        municipality: String,
        location: (f64, f64),
        love_action_level: i32,
        amount: f64,
        action_content: String,
        sender_public_key: String,
        receiver_public_key: String,
    ) -> Self {
        let mut transaction = Transaction {
            transaction_id,
            municipality,
            timestamp: Utc::now(),
            location,
            love_action_level,
            amount,
            action_content,
            is_local: true,
            close_flag: false,
            approval_target: None,
            sender_public_key,
            receiver_public_key,
            signature: Vec::new(),
            location_hash: Vec::new(),
            received_timestamp: None,
            recipient_location: None,
            fee: 0.0,
        };
        transaction.calculate_location_hash();
        transaction.generate_signature();
        transaction
    }

    pub fn calculate_location_hash(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", self.location).as_bytes());
        self.location_hash = hasher.finalize().to_vec();
    }

    pub fn generate_proof_of_history(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}{:?}", self.transaction_id, self.timestamp).as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn generate_signature(&mut self) {
        let message = format!(
            "{}{}{}{}{}{}{}{}{}{}",
            self.transaction_id,
            self.municipality,
            self.timestamp.to_rfc3339(),
            format!("{:?}", self.location),
            self.love_action_level,
            self.amount,
            self.action_content,
            hex::encode(&self.location_hash),
            self.sender_public_key,
            self.receiver_public_key
        );
        self.signature = ntru_sign(message.as_bytes(), &hex::decode(&self.sender_public_key).unwrap());
    }

    pub fn verify_signature(&self) -> bool {
        let message = format!(
            "{}{}{}{}{}{}{}{}{}{}",
            self.transaction_id,
            self.municipality,
            self.timestamp.to_rfc3339(),
            format!("{:?}", self.location),
            self.love_action_level,
            self.amount,
            self.action_content,
            hex::encode(&self.location_hash),
            self.sender_public_key,
            self.receiver_public_key
        );
        verify_signature(
            message.as_bytes(),
            &self.signature,
            &hex::decode(&self.sender_public_key).unwrap(),
        )
    }
}

// Consensusの実装
pub struct Consensus {
    pub dpos: DPoS,
    pub poh: ProofOfHistory,
    pub transactions: Vec<Transaction>,
}

impl Consensus {
    pub fn new(municipalities: Vec<String>) -> Self {
        Consensus {
            dpos: DPoS::new(municipalities),
            poh: ProofOfHistory::new(),
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn process_transactions(&mut self) {
        for transaction in &mut self.transactions {
            if self.dpos.approve_transaction(transaction).is_ok() {
                self.poh.add_event(&transaction.generate_proof_of_history());
                println!("Transaction processed: {:?}", transaction);
            } else {
                println!("Transaction failed: {:?}", transaction);
            }
        }
    }

    pub fn generate_poh_hash(&self) -> String {
        self.poh.generate_hash()
    }
}
