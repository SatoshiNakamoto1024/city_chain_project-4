#[macro_use] extern crate rocket;
#[macro_use] extern crate serde;

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::http::Status;
use rocket::config::{Config, Environment, TlsConfig};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::sync::Arc;
use chrono::Utc;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::process::Command;
use my_ntru_lib::{NTRUKeys, generate_ntru_keys, ntru_encrypt, ntru_decrypt, sign_transaction, verify_signature};

#[derive(Serialize, Deserialize, Clone)]
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    prev_hash: String,
    hash: String,
    verifiable_credential: String,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f64,
    verifiable_credential: String,
    signature: Vec<u8>,
}

type Blockchain = Arc<Mutex<Vec<Block>>>;

#[post("/transaction", format = "json", data = "<transaction>")]
async fn create_transaction(transaction: Json<Transaction>, client: &rocket::State<Client>) -> Json<Transaction> {
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
        }),
    }
}

#[post("/add_block", format = "json", data = "<block>")]
async fn add_block(block: Json<Block>, chain: &rocket::State<Blockchain>, client: &rocket::State<Client>) -> Status {
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
async fn get_chain(chain: &rocket::State<Blockchain>) -> Json<Vec<Block>> {
    let chain = chain.lock().await;
    Json(chain.clone())
}

#[rocket::main]
async fn main() {
    let chain = Arc::new(Mutex::new(Vec::<Block>::new()));

    // SSL設定
    let tls_config = TlsConfig::from_paths("cert.pem", "key.pem");
    let config = Config::figment()
        .merge(("tls.certs", "cert.pem"))
        .merge(("tls.key", "key.pem"));

    rocket::custom(config)
        .manage(chain)
        .manage(Client::new())
        .mount("/", routes![create_transaction, add_block, get_chain])
        .attach(AdHoc::on_ignite("SSL Config", |rocket| async {
            rocket.config().await.unwrap();
            rocket
        }))
        .launch()
        .await
        .unwrap();
}
