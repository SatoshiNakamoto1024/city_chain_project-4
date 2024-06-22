#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::sync::Arc;
use chrono::Utc;
use reqwest::Client;

#[derive(Serialize, Deserialize, Clone)]
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    prev_hash: String,
    hash: String,
}

#[derive(Serialize, Deserialize)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f64,
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
        }),
    }
}

#[post("/add_block", format = "json", data = "<block>")]
async fn add_block(block: Json<Block>, chain: &rocket::State<Blockchain>, client: &rocket::State<Client>) -> Status {
    let mut chain = chain.lock().await;
    // ブロックの検証と追加
    chain.push(block.into_inner());

    // グローバルメインチェーンへのブロック転送
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
    let ssl = SslAcceptor::mozilla_intermediate(SslMethod::tls())
        .unwrap()
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap()
        .set_certificate_chain_file("cert.pem")
        .unwrap();
    
    rocket::build()
        .manage(chain)
        .manage(Client::new())
        .mount("/", routes![create_transaction, add_block, get_chain])
        .attach(AdHoc::on_ignite("SSL Config", |rocket| async {
            rocket::config::Config {
                tls: Some(rocket::config::TlsConfig::new(ssl)),
                ..rocket::config::Config::default()
            }
        }))
        .launch()
        .await
        .unwrap();
}
