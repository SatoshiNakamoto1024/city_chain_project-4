#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::sync::Arc;
use chrono::Utc;

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
async fn create_transaction(transaction: Json<Transaction>, chain: &rocket::State<Blockchain>) -> Json<Transaction> {
    // トランザクション作成ロジック
    let mut chain = chain.lock().await;
    // トランザクションを含む新しいブロックを作成
    let new_block = Block {
        index: chain.len() as u64 + 1,
        timestamp: Utc::now().timestamp() as u64,
        data: serde_json::to_string(&*transaction).unwrap(),
        prev_hash: chain.last().map_or("0".to_string(), |b| b.hash.clone()),
        hash: "some_hash".to_string(), // 実際にはハッシュを計算する必要があります
    };
    chain.push(new_block);

    transaction
}

#[post("/add_block", format = "json", data = "<block>")]
async fn add_block(block: Json<Block>, chain: &rocket::State<Blockchain>) -> Status {
    let mut chain = chain.lock().await;
    // ブロックの検証と追加
    chain.push(block.into_inner());
    Status::Accepted
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
