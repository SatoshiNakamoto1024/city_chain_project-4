#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::http::Status;
use rocket::config::{Config, Environment, TlsConfig};
use std::sync::Arc;
use chrono::Utc;
use reqwest::Client;
use ntru::{NtruEncrypt, NtruSign, NtruParam};
use rand::rngs::OsRng;

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
    chain.push(block.into_inner());

    let global_chain_url = "http://global_main_chain:8000/add_block";
    let res = client.post(global_chain_url)
                    .json(&*block)
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
        .launch()
        .await
        .unwrap();
}
