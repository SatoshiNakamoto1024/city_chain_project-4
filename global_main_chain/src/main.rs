#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::http::Status;
use rocket::config::{Config, TlsConfig};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rand::{Rng, prelude::SliceRandom};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use mongodb::{Client as MongoClient, options::ClientOptions, Collection, bson::{doc, Bson, DateTime as BsonDateTime}};
use futures::stream::TryStreamExt;
use immudb_proto::immudb_proto_function;

use ntru::ntru_encrypt::NtruEncrypt;
use ntru::ntru_sign::NtruSign;
use ntru::ntru_param::NtruParam;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    index: u64,
    timestamp: DateTime<Utc>,
    data: String,
    prev_hash: String,
    hash: String,
    verifiable_credential: String,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    transaction_id: String,
    sender: String,
    receiver: String,
    amount: f64,
    verifiable_credential: String,
    signature: Vec<u8>,
    location: (f64, f64),
    timestamp: String,
    proof_of_place: String,
    sender_municipality: String,
    receiver_municipality: String,
    sender_continent: String,
    receiver_continent: String,
    sender_municipal_id: String,
    receiver_municipal_id: String,
    status: String,
    created_at: DateTime<Utc>,
}

impl Transaction {
    fn new(
        sender: String,
        receiver: String,
        amount: f64,
        verifiable_credential: String,
        location: (f64, f64),
        sender_municipality: String,
        receiver_municipality: String,
        sender_continent: String,
        receiver_continent: String,
        sender_municipal_id: String,
        receiver_municipal_id: String,
    ) -> Self {
        let mut transaction = Transaction {
            transaction_id: uuid::Uuid::new_v4().to_string(),
            sender,
            receiver,
            amount,
            verifiable_credential,
            signature: vec![],
            location,
            timestamp: Utc::now().to_rfc3339(),
            proof_of_place: String::new(),
            sender_municipality,
            receiver_municipality,
            sender_continent,
            receiver_continent,
            sender_municipal_id,
            receiver_municipal_id,
            status: "pending".to_string(),
            created_at: Utc::now(),
        };
        transaction.proof_of_place = ProofOfPlace::new(location).generate_proof();
        transaction
    }

    fn generate_proof_of_history(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}{:?}", self.sender, self.timestamp).as_bytes());
        hex::encode(hasher.finalize())
    }

    fn verify_signature(&self, public_key: &[u8]) -> bool {
        let ntru_param = NtruParam::default(); // 適切なパラメータを選択
        let ntru_sign = NtruSign::new(&ntru_param);
    
        let transaction_data = format!("{:?}", self);
        
        // 公開鍵を使用して署名を検証
        match ntru_sign.verify(&transaction_data.as_bytes(), &self.signature, &public_key) {
            Ok(true) => true,
            _ => false,
        }
    }    
}

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
            let ntru_param = NtruParam::default(); // 適切なパラメータを選択
            let ntru_sign = NtruSign::new(&ntru_param);
            
            // トランザクションデータを署名
            let transaction_data = format!("{:?}", transaction);
            let private_key = // ここで秘密鍵を適切に取得する
            let signature = ntru_sign.sign(&transaction_data.as_bytes(), &private_key).expect("Signing failed");
            
            transaction.signature = signature;
            
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

    fn add_transaction(&mut self, mut transaction: Transaction) {
        if self.dpos.approve_transaction(&mut transaction).is_ok() {
            self.poh.add_event(&transaction.generate_proof_of_history());
            println!("Transaction processed: {:?}", transaction);
            self.transactions.push(transaction);
        } else {
            println!("Transaction approval failed.");
        }
    }

    fn generate_poh_hash(&self) -> String {
        self.poh.generate_hash()
    }
}

#[derive(Clone)]
struct AppState {
    mongo_collection: Collection<Transaction>,
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to the Global Main Chain!"
}

#[post("/transaction", format = "json", data = "<transaction_json>")]
async fn create_transaction(
    transaction_json: Json<Transaction>,
    client: &rocket::State<Client>,
    state: &rocket::State<Arc<AppState>>,
) -> Result<Json<Transaction>, Status> {
    let mut transaction = transaction_json.into_inner();

    // 送信者の公開鍵を取得（ここでは仮に公開鍵が設定されているとします）
    let public_key = // ここで公開鍵を適切に取得する（例: データベースやファイルシステムから取得）

    // 署名を検証
    if !transaction.verify_signature(&public_key) {
        println!("Invalid signature for transaction: {:?}", transaction.transaction_id);
        return Err(Status::BadRequest);
    }

    // DPoS コンセンサスの処理
    let mut consensus = Consensus::new(vec![
        transaction.sender_municipality.clone(),
        transaction.receiver_municipality.clone(),
    ]);

    consensus.add_transaction(transaction.clone());

    // トランザクションをデータベースに保存
    let state_clone = state.clone();
    match state_clone.mongo_collection.insert_one(transaction.clone(), None).await {
        Ok(_) => {
            println!("Transaction inserted into MongoDB.");
        }
        Err(e) => {
            println!("Failed to insert transaction into MongoDB: {:?}", e);
            return Err(Status::InternalServerError);
        }
    }

    // トランザクションをクライアントに返す
    Ok(Json(transaction))
}

#[post("/receive_block", format = "json", data = "<block>")]
async fn receive_block(
    block: Json<Block>,
    chain: &rocket::State<Blockchain>,
) -> Status {
    println!("Received block from continent: {:?}", block);

    let mut chain = chain.lock().await;
    chain.push(block.into_inner());

    println!("Block added to global chain.");

    Status::Accepted
}

#[post("/add_block", format = "json", data = "<block_json>")]
async fn add_block(
    block_json: Json<Block>,
    chain: &rocket::State<Blockchain>,
    client: &rocket::State<Client>,
) -> Status {
    println!("Received block: {:?}", block_json);
    let mut chain = chain.lock().await;
    let block_inner = block_json.into_inner();
    chain.push(block_inner);

    println!("Block added to local chain.");

    // グローバルチェーンへのブロック送信処理を実装する必要があります

    Status::Accepted
}

#[get("/chain")]
async fn get_chain(chain: &rocket::State<Blockchain>) -> Json<Vec<Block>> {
    let chain = chain.lock().await;
    Json(chain.clone())
}

#[rocket::main]
async fn main() {
    immudb_proto_function();

    // MongoDB クライアントの初期化
    let mongo_client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let mongo_client = MongoClient::with_options(mongo_client_options).unwrap();
    let mongo_db = mongo_client.database("global_chain_db");
    let mongo_collection = mongo_db.collection::<Transaction>("transactions");

    // インデックスの作成
    mongo_collection.create_index(
        mongodb::IndexModel::builder()
            .keys(doc! { "transaction_id": 1 })
            .build(),
        None,
    ).await.unwrap();

    // アプリケーション状態の初期化
    let app_state = Arc::new(AppState {
        mongo_collection,
    });

    let chain = Arc::new(Mutex::new(Vec::<Block>::new()));

    let tls_config = TlsConfig::from_paths(
        "D:\\city_chain_project\\cert.crt",
        "D:\\city_chain_project\\key.pem",
    );
    let config = Config::figment()
        .merge(("tls.certs", "D:\\city_chain_project\\cert.crt"))
        .merge(("tls.key", "D:\\city_chain_project\\key.pem"));

    rocket::custom(config)
        .manage(chain)
        .manage(Client::new())
        .manage(app_state)
        .mount("/", routes![index, create_transaction, add_block, get_chain])
        .launch()
        .await
        .unwrap();
}
