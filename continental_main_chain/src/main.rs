// 必要なモジュールのインポート
#[macro_use] extern crate rocket;
use rocket::{get, post, routes, serde::json::Json, tokio::sync::Mutex, State};
use rocket::serde::{Deserialize, Serialize};
use rocket::http::Status;
use rocket::config::{Config, TlsConfig};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rand::{Rng, prelude::SliceRandom};
use sha2::{Sha256, Digest};
use hex;

use ntru::ntru_encrypt::NtruEncrypt;
use ntru::ntru_sign::NtruSign;
use ntru::ntru_param::NtruParam;

#[derive(Serialize, Deserialize, Debug, Clone)] // Debug を追加
struct Block {
    index: u64,
    timestamp: DateTime<Utc>,
    data: String,
    prev_hash: String,
    hash: String,
    verifiable_credential: String,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)] // Debug を追加
struct Transaction {
    sender: String,
    receiver: String,
    amount: f64,
    verifiable_credential: String,
    signature: Vec<u8>,
    transaction_id: String,
    timestamp: String,
}

// Blockchain型の定義
type Blockchain = Arc<Mutex<Vec<Block>>>;

struct DPoS {
    municipalities: Vec<String>,
    approved_representative: Option<String>,
}

impl DPoS {
    fn new(municipalities: Vec<String>) -> Self {
        Self {
            municipalities,
            approved_representative: None,
        }
    }

    fn elect_representative(&mut self) -> String {
        let representative = self.municipalities.choose(&mut rand::thread_rng()).unwrap().clone();
        self.approved_representative = Some(representative.clone());
        representative
    }

    fn approve_transaction(&self, transaction: &mut Transaction) -> Result<&str, &str> {
        if let Some(representative) = &self.approved_representative {
            transaction.signature = format!("approved_by_{}", representative).as_bytes().to_vec();
            Ok("Transaction approved")
        } else {
            Err("No representative elected")
        }
    }
}

struct ProofOfPlace {
    location: (f64, f64),
    timestamp: DateTime<Utc>,
}

impl ProofOfPlace {
    fn new(location: (f64, f64)) -> Self {
        ProofOfPlace {
            location,
            timestamp: Utc::now(),
        }
    }

    fn generate_proof(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}{:?}", self.location, self.timestamp).as_bytes());
        hex::encode(hasher.finalize())
    }

    fn verify_proof(proof: &str, location: (f64, f64), timestamp: DateTime<Utc>) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}{:?}", location, timestamp).as_bytes());
        let computed_proof = hex::encode(hasher.finalize());
        computed_proof == proof
    }
}

struct ProofOfHistory {
    sequence: Vec<String>,
}

impl ProofOfHistory {
    fn new() -> Self {
        ProofOfHistory { sequence: Vec::new() }
    }

    fn add_event(&mut self, event: &str) {
        self.sequence.push(event.to_string());
    }

    fn generate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for event in &self.sequence {
            hasher.update(event.as_bytes());
        }
        hex::encode(hasher.finalize())
    }
}

struct Consensus {
    dpos: DPoS,
    poh: ProofOfHistory,
    transactions: Vec<Transaction>,
}

impl Consensus {
    fn new(municipalities: Vec<String>) -> Self {
        Consensus {
            dpos: DPoS::new(municipalities),
            poh: ProofOfHistory::new(),
            transactions: Vec::new(),
        }
    }

    fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    fn process_transactions(&mut self) {
        for transaction in &mut self.transactions {
            if self.dpos.approve_transaction(transaction).is_ok() {
                self.poh.add_event(&transaction.generate_proof_of_history());
                println!("Transaction processed: {:?}", transaction);
            } else {
                println!("Transaction failed: {:?}", transaction);
            }
        }
    }

    fn generate_poh_hash(&self) -> String {
        self.poh.generate_hash()
    }
}

impl Transaction {
    fn generate_proof_of_history(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}{:?}", self.sender, self.timestamp).as_bytes());
        hex::encode(hasher.finalize())
    }
}

// 新しくルートを追加
#[get("/")]
fn index() -> &'static str {
    "Welcome to the Continental Main Chain!"
}

#[post("/transaction", format = "json", data = "<transaction>")]
async fn create_transaction(transaction: Json<Transaction>, client: &State<Client>) -> Json<Transaction> {
    // トランザクション作成ロジック
    let global_chain_url = "http://global_main_chain:8000/transaction";
    let res = client.post(global_chain_url)
                    .json(&*transaction)
                    .send()
                    .await;
    
    match res {
        Ok(_) => transaction,
        Err(_) => Json(Transaction {
            sender: "error".to_string(),
            receiver: "error".to_string(),
            amount: 0.0,
            verifiable_credential: "error".to_string(),
            signature: vec![],
            transaction_id: "error".to_string(),
            timestamp: "error".to_string(),
        }),
    }
}

#[post("/add_block", format = "json", data = "<block>")]
async fn add_block(block: Json<Block>, chain: &State<Blockchain>, client: &State<Client>) -> Status {
    let mut chain = chain.lock().await;
    let block = block.into_inner();

    // ラティス署名の検証
    if !verify_signature(&block.data.as_bytes(), &block.signature, &block.verifiable_credential.as_bytes()) {
        return Status::Forbidden;
    }
    
    // ブロックの検証と追加
    chain.push(block.clone());

    // グローバルメインチェーンへのブロック転送
    let global_chain_url = "http://global_main_chain:8000/add_block";
    let res = client.post(global_chain_url)
                    .json(&block)
                    .send()
                    .await;

    match res {
        Ok(_) => Status::Accepted,
        Err(_) => Status::InternalServerError,
    }
}

#[get("/chain")]
async fn get_chain(chain: &State<Blockchain>) -> Json<Vec<Block>> {
    let chain = chain.lock().await;
    Json(chain.clone())
}

// 署名の検証関数の実装を追加
fn verify_signature(data: &[u8], signature: &[u8], credential: &[u8]) -> bool {
    // 実際の署名検証ロジックをここに実装する
    true
}

#[rocket::main]
async fn main() {
    let chain = Arc::new(Mutex::new(Vec::<Block>::new()));

    // SSL設定
    let tls_config = TlsConfig::from_paths("cert.crt", "key.pem");
    let config = Config::figment()
        .merge(("tls.certs", "cert.crt"))
        .merge(("tls.key", "key.pem"));
        
    rocket::custom(config)
        .manage(chain)
        .manage(Client::new())
        .mount("/", routes![index, create_transaction, add_block, get_chain]) // インデックスルートをマウント
        .launch()
        .await
        .unwrap();
}
