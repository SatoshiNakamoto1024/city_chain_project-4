use rocket::{launch, Rocket, Build};
use rocket::post;  // postマクロをインポート
use rocket::get;  // getマクロをインポート
use rocket::routes;  // routesマクロをインポート
use rocket::http::Status;
use rocket::State;
use rocket::fairing::AdHoc;
use rocket::config::{Config, TlsConfig};
use base64::encode;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::serde::json::Value; // `Value` を `rocket::serde` のものに統一
use std::sync::Arc; 
use std::fs;  // fsモジュールをインポート
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::env;  // ここに `std::env` をインポート
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::runtime::Runtime;
use tokio::time::Duration;  // tokio を使用している場合
use tokio::sync::Mutex; 
use tokio_stream::StreamExt;  // これを追加
use chrono::format::ParseError;
use chrono::{DateTime, Utc, Duration, TimeZone};
use chrono::NaiveDateTime;
use reqwest::Client;
use rand::prelude::*;
use rand::seq::SliceRandom;
use sha2::{Sha256, Digest};
use mongodb::{Client as MongoClient, options::ClientOptions, Collection};  // `Collection` を適切にインポート
use mongodb::bson::{doc, Document, DateTime as BsonDateTime};  // BSON関連のインポートに統一
use mongodb::options::IndexOptions; // 追加
use mongodb::IndexModel; // 追加
use mongodb::results::UpdateResult;
use mongodb::error::Error as MongoError;
use uuid::Uuid;
use ntru::ntru_encrypt::NtruEncrypt;
use ntru::ntru_sign::NtruSign;
use ntru::ntru_param::NtruParam;
use immudb_proto::{SetRequest, KeyValue};
use immudb_proto::GetRequest;
use immudb_proto::immudb_proto_function;
use prost::bytes::Bytes;

// 評価項目を保持する構造体
struct EvaluationItems {
    total_love_token_usage: f64,
    value_from_tokens_received: f64,
    contribution_by_token_usage: f64,
}
// ユーザーごとの評価データを保持するハッシュマップ
type UserEvaluations = Arc<Mutex<HashMap<String, EvaluationItems>>>;

// 代表者情報を保持する構造体
struct Representative {
    user_id: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}
// 市町村ごとの代表者リスト
type MunicipalRepresentatives = Arc<Mutex<HashMap<String, Vec<Representative>>>>;

// municipalities.jsonファイルをロードする関数
fn load_municipalities_data() -> Result<Value, serde_json::Error> {
    let file_path = "D:\\city_chain_project\\dapps\\municipalities.json"; // 動的にパスを取得するか、環境変数で設定するように変更
    let file_content = std::fs::read_to_string(file_path).expect("Failed to open municipalities.json");
    serde_json::from_str(&file_content)
}

// 大陸名と市町村名に基づいてポートとMongoDB URIを取得する関数
fn get_city_ports_and_uri(continent_city: &str) -> Result<(u16, String), String> {
    let config_path = "D:\\city_chain_project\\dapps\\municipalities.json";  // 設定ファイルへのパスを指定
    let file = File::open(config_path).map_err(|e| format!("Failed to open config file: {}", e))?;
    let reader = BufReader::new(file);
    let config: Value = serde_json::from_reader(reader).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // 大陸と市町村を分割
    let parts: Vec<&str> = continent_city.split('-').collect();
    if parts.len() != 2 {
        return Err("Invalid argument format. Expected format: Continent-City".to_string());
    }
    let continent = parts[0];
    let city = parts[1];

    // 設定ファイルからフラスクポートとMongoDB URIを取得
    if let Some(continent_data) = config.get(continent) {
        if let Some(cities) = continent_data.get("cities").and_then(|c| c.as_array()) {
            for city_data in cities {
                if city_data.get("name").and_then(|n| n.as_str()) == Some(city) {
                    let flask_port = city_data.get("city_flask_port")
                        .and_then(|p| p.as_str())
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(1034);
                    let mongo_port = city_data.get("city_port")
                        .and_then(|p| p.as_str())
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(10034);
                    let mongo_uri = format!("mongodb://localhost:{}", mongo_port);
                    println!("Using configuration for {}-{}: Flask Port: {}, Mongo URI: {}", continent, city, flask_port, mongo_uri);  // デバッグ用出力
                    return Ok((flask_port, mongo_uri));
                }
            }
        }
    }

    // 設定が見つからなかった場合にデフォルトポートを返す
    println!("Loaded config: {:?}", config);
    println!("Looking for continent: {}, city: {}", continent, city);

    let default_flask_port = 1034;
    let default_mongo_port = 10034;
    let default_mongo_uri = format!("mongodb://localhost:{}", default_mongo_port);
    println!("Configuration for {}-{} not found. Using default settings: Flask Port: {}, Mongo URI: {}", continent, city, default_flask_port, default_mongo_uri);  // デバッグ用出力
    Ok((default_flask_port, default_mongo_uri))
}

// MongoDB設定を読み込む関数
fn load_mongodb_config() -> Result<Value, Box<dyn std::error::Error>> {
    let file_path = "D:\\city_chain_project\\mongodb_config.json";
    let file_content = fs::read_to_string(file_path)?;
    let json_data: Value = serde_json::from_str(&file_content)?;
    Ok(json_data)
}

fn get_mongo_uri(instance_type: &str, continent: &str) -> String {
    let mongodb_config = load_mongodb_config().expect("Failed to load MongoDB config");
    
    if let Some(instance_config) = mongodb_config.get(instance_type) {
        if let Some(uri) = instance_config.get(continent) {
            return uri.as_str().unwrap().to_string();
        }
        if let Some(default_uri) = instance_config.get("default") {
            return default_uri.as_str().unwrap().to_string();
        }
    }
    panic!("MongoDB URI not found for instance type '{}' and continent '{}'", instance_type, continent);
}

fn get_other_continental_chains(municipalities_data: &Value, current_continent_city: &str) -> Vec<String> {
    municipalities_data.as_object()
        .expect("Failed to convert municipalities data to object")
        .iter()
        .filter(|(key, _)| key.as_str() != current_continent_city.split('-').next().unwrap_or("Default"))
        .map(|(_, data)| {
            let flask_port = data.get("flask_port").and_then(|p| p.as_str()).unwrap_or("1070");
            format!("http://127.0.0.1:{}", flask_port)
        })
        .collect()
}

// タイムスタンプを解析する関数
fn parse_timestamp(timestamp: &str) -> Result<DateTime<Utc>, ParseError> {
    println!("Received timestamp: {}", timestamp);
    
    // "Z" の有無をチェックし、必要ならば取り除いて処理
    let cleaned_timestamp = timestamp.trim_end_matches('Z');

    let naive_datetime = NaiveDateTime::parse_from_str(cleaned_timestamp, "%Y-%m-%dT%H:%M:%S%.f%:z")?;
    let datetime_utc = DateTime::<Utc>::from_utc(naive_datetime, Utc);
    Ok(datetime_utc)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
struct Block {
    index: u64,
    timestamp: DateTime<Utc>,
    data: String,
    prev_hash: String,
    hash: String,
    verifiable_credential: String,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
struct Transaction {
    transaction_id: String,
    sender: String,
    receiver: String,
    amount: f64,
    verifiable_credential: String,
    signature: String,
    location: String,
    timestamp: DateTime<Utc>,
    proof_of_place: String,
    subject: String,
    action_level: String,
    dimension: String,
    fluctuation: String,
    organism_name: String,
    details: String,
    goods_or_money: String,
    transaction_type: String,  // 新しく追加するフィールド
    sender_municipality: String, // 追加
    receiver_municipality: String, // 追加
    sender_continent: String,  // 新しく追加
    receiver_continent: String,  // 新しく追加
    status: String,  // 新しく追加
    created_at: DateTime<Utc>,  // 新しく追加
    sender_municipal_id: String,   // 追加
    receiver_municipal_id: String, // 追加
}

type Blockchain = Arc<Mutex<Vec<Block>>>;

type PendingTransactions = Arc<Mutex<HashMap<String, Transaction>>>;

// AppState構造体を定義
#[derive(Clone)]
struct AppState {
    pub pending_transactions: Arc<Mutex<HashMap<String, Transaction>>>,
    pub blockchain: Arc<Mutex<Vec<Block>>>,
    pub mongo_collection: Collection<Transaction>,
    pub other_continental_chains: Arc<Mutex<Vec<String>>>, // この部分を変更
    pub dpos: Arc<Mutex<DPoS>>,
    pub user_evaluations: UserEvaluations,             // ユーザーの評価データ
    pub municipal_representatives: MunicipalRepresentatives, // 市町村ごとの代表者リスト
}

// Gossipによってブロックを他のノードと共有するためのリクエストデータ
#[derive(Serialize, Deserialize, Debug)]
struct GossipBlockRequest {
    blocks: Vec<Block>,  // ブロックのリスト
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GossipRequest {
    transactions: HashMap<String, Transaction>, // トランザクションIDとトランザクションデータのペア
}

#[derive(Debug, Deserialize)]
struct ImmudbConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct Config {
    municipalities: HashMap<String, ImmudbConfig>,
    default: ImmudbConfig,
}

#[derive(Debug, Deserialize)]
struct MongoConfig {
    send_pending: HashMap<String, String>,
    receive_pending: HashMap<String, String>,
    analytics: HashMap<String, String>,
    update_status_url: String,
}

fn load_mongo_config() -> Result<MongoConfig, Box<dyn std::error::Error>> {
    let file = File::open("mongo_config.json")?;
    let reader = BufReader::new(file);
    let config: MongoConfig = serde_json::from_reader(reader)?;
    Ok(config)
}

fn load_immudb_config() -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open("immudb_config.json")?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    Ok(config)
}

fn get_immudb_address(config: &Config, municipality: &str) -> (String, u16) {
    if let Some(immudb_config) = config.municipalities.get(municipality) {
        (immudb_config.host.clone(), immudb_config.port)
    } else {
        (config.default.host.clone(), config.default.port)
    }
}
struct DPoS {
    user_evaluations: HashMap<String, EvaluationItems>, // ユーザーの評価データ
    municipal_representatives: HashMap<String, Vec<Representative>>, // 市町村ごとの代表者リスト
    approved_representative: Option<String>,
}

type SharedDPoS = Arc<Mutex<DPoS>>;

impl AppState {
    async fn create_indexes(&self) {
        // "status" フィールドのインデックスを作成
        let status_index = IndexModel::builder()
            .keys(doc! { "status": 1 })
            .options(Some(IndexOptions::builder().build()))
            .build();

        self.mongo_collection
            .create_index(status_index, None)
            .await
            .expect("Failed to create index on status");

        // "created_at" フィールドのインデックスを作成
        let created_at_index = IndexModel::builder()
            .keys(doc! { "created_at": 1 })
            .options(Some(IndexOptions::builder().build()))
            .build();

        self.mongo_collection
            .create_index(created_at_index, None)
            .await
            .expect("Failed to create index on created_at");

        println!("Indexes created on 'status' and 'created_at' fields.");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvaluationItems {
    total_love_token_usage: f64,
    value_from_tokens_received: f64,
    contribution_by_token_usage: f64,
}

#[derive(Debug, Clone)]
struct Representative {
    user_id: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

struct DPoS {
    user_evaluations: HashMap<String, EvaluationItems>, // ユーザーの評価データ
    municipal_representatives: HashMap<String, Vec<Representative>>, // 市町村ごとの代表者リスト
    approved_representative: Option<String>,
}

impl DPoS {
    /// 新しい DPoS インスタンスを作成
    fn new() -> Self {
        println!("Initializing DPoS...");
        DPoS {
            user_evaluations: HashMap::new(),
            municipal_representatives: HashMap::new(),
            approved_representative: None,
        }
    }

    /// トランザクションに基づいてユーザーの評価データを更新
    async fn update_user_evaluations(&mut self, transaction: &Transaction) {
        // 送信者の評価データを更新
        let sender_eval = self.user_evaluations.entry(transaction.sender.clone()).or_insert(EvaluationItems {
            total_love_token_usage: 0.0,
            value_from_tokens_received: 0.0,
            contribution_by_token_usage: 0.0,
        });
        sender_eval.total_love_token_usage += transaction.amount;
        sender_eval.contribution_by_token_usage += self.calculate_contribution(transaction);

        // 受信者の評価データを更新
        let receiver_eval = self.user_evaluations.entry(transaction.receiver.clone()).or_insert(EvaluationItems {
            total_love_token_usage: 0.0,
            value_from_tokens_received: 0.0,
            contribution_by_token_usage: 0.0,
        });
        receiver_eval.value_from_tokens_received += transaction.amount;
    }

    /// 愛貨消費による貢献度を計算する（仮の実装）
    fn calculate_contribution(&self, transaction: &Transaction) -> f64 {
        // 実際のロジックをここに実装
        transaction.amount // 仮にトランザクションの金額をそのまま返す
    }

    /// 3ヶ月ごとに代表者を選出する関数
    fn select_representatives(&mut self) {
        let now = Utc::now();
        let day = now.day();
        let month = now.month();

        // 代表者選出の日付をチェック（例：7月1日、10月1日、1月1日、4月1日）
        if day == 1 && (month == 7 || month == 10 || month == 1 || month == 4) {
            // 評価期間の開始と終了を計算
            let evaluation_end = now - Duration::days(1); // 前日まで
            let evaluation_start = evaluation_end - Duration::days(90); // 90日前

            // 代表者の選出を実行
            self.perform_selection(evaluation_start, evaluation_end);
        }
    }

    /// 代表者の選出を実行する関数
    fn perform_selection(&mut self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) {
        // ユーザー評価データを期間内のデータにフィルタリングする必要があるが、ここでは簡略化のため全体を使用

        // 市町村ごとにユーザーをグループ化してランキング
        let mut municipal_rankings: HashMap<String, Vec<(String, f64)>> = HashMap::new();

        for (user_id, eval) in &self.user_evaluations {
            // ユーザーの市町村を取得（この関数は実装が必要）
            let municipality = self.get_user_municipality(user_id);

            let total_score = eval.total_love_token_usage + eval.value_from_tokens_received + eval.contribution_by_token_usage;

            municipal_rankings.entry(municipality).or_insert(Vec::new()).push((user_id.clone(), total_score));
        }

        // 市町村ごとに上位5名を選出
        for (municipality, mut rankings) in municipal_rankings {
            // スコアの高い順にソート
            rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            // 上位5名を選出
            let top_5 = rankings.into_iter().take(5);

            let representatives = top_5.map(|(user_id, _)| Representative {
                user_id,
                start_date: end_date + Duration::days(90), // 任期開始は3ヶ月後
                end_date: end_date + Duration::days(90 * 2), // 任期はその3ヶ月後まで
            }).collect::<Vec<_>>();

            self.municipal_representatives.insert(municipality, representatives);
        }

        // 新しい代表者へのメッセージ送信（実装が必要）
        for reps in self.municipal_representatives.values() {
            for rep in reps {
                if rep.start_date == end_date + Duration::days(90) {
                    self.send_message_to_user(&rep.user_id, "10月1日～12月31日までの期間、代表よろしくお願いします");
                }
            }
        }
    }

    /// ユーザーIDから市町村を取得する関数（実装が必要）
    fn get_user_municipality(&self, user_id: &str) -> String {
        // 例としてユーザーIDから市町村を推測
        // ユーザーIDが "Asia-Tokyo-User1" のような形式であると仮定
        let parts: Vec<&str> = user_id.split('-').collect();
        if parts.len() >= 2 {
            format!("{}-{}", parts[0], parts[1])
        } else {
            "Unknown-Municipality".to_string()
        }
    }

    /// 市町村の代表者を選出する関数
    async fn elect_representative(&mut self, municipality: &str) {
        println!("Electing representative for municipality: {}", municipality);

        // 現在の日時を取得
        let now = Utc::now();

        if let Some(representatives) = self.municipal_representatives.get(municipality) {
            // 任期中の代表者をフィルタリング
            let active_reps: Vec<&Representative> = representatives.iter().filter(|rep| rep.start_date <= now && now <= rep.end_date).collect();

            if !active_reps.is_empty() {
                // 任期中の代表者からランダムに選出
                if let Some(rep) = active_reps.choose(&mut rand::thread_rng()) {
                    self.approved_representative = Some(rep.user_id.clone());
                    println!("代表者が選ばれました: {:?}", rep.user_id);
                } else {
                    println!("代表者選出に失敗しました: {:?}", municipality);
                }
            } else {
                println!("任期中の代表者が見つかりません: {}", municipality);
            }
        } else {
            println!("市町村の代表者リストが存在しません: {}", municipality);
        }
    }

    /// トランザクションを承認する関数
    async fn approve_transaction(&self, transaction: &Transaction) -> bool {
        if let Some(representative) = &self.approved_representative {
            // 送信者の市町村の代表者リストを取得
            let sender_municipality = transaction.sender_municipality.clone();
            if let Some(representatives) = self.municipal_representatives.get(&sender_municipality) {
                // 任期中の代表者かどうかを確認
                let now = Utc::now();
                let is_valid = representatives.iter().any(|rep| rep.user_id == *representative && rep.start_date <= now && now <= rep.end_date);
                if is_valid {
                    println!("トランザクション承認: 代表者は {}", representative);
                    return true;
                } else {
                    println!("トランザクション拒否: 代表者がリストに含まれていません。");
                    return false;
                }
            } else {
                println!("市町村の代表者リストが存在しません: {}", sender_municipality);
                return false;
            }
        } else {
            println!("トランザクション拒否: 代表者が選ばれていません。");
            false
        }
    }

    /// ユーザーにメッセージを送信する関数（実装が必要）
    fn send_message_to_user(&self, user_id: &str, message: &str) {
        // メッセージ送信ロジックを実装
        println!("メッセージ送信先: {} 内容: {}", user_id, message);
    }

    /// 代表者の交代や繰り上げを処理する関数
    fn handle_representative_changes(&mut self) {
        // 任期中に代表者が辞任した場合の処理を実装
        // 辞任した代表者をリストから削除し、次点のユーザーを繰り上げる
    }

    /// 大陸代表者を選出する関数（実装が必要）
    fn select_continental_representatives(&mut self) {
        // 市町村代表者から大陸代表者を選出するロジックを実装
    }

    /// ブロックを承認する関数
    fn approve_block(&self, block: &Block) -> bool {
        // 大陸代表者によるブロック承認のロジックを実装
        // ここでは仮に常に承認するとする
        println!("ブロック承認: 代表者は {:?}", self.approved_representative);
        true
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
    fn new(municipalities_data: &serde_json::Value) -> Self {
        Consensus {
            dpos: DPoS::new(municipalities_data),  // municipalities_dataを渡す
            poh: ProofOfHistory::new(),
            transactions: Vec::new(),
        }
    }

    fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    fn process_transactions(&mut self) {
        for transaction in &self.transactions {
            if self.dpos.approve_transaction(transaction) {
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
    // Transaction構造体の署名を検証するメソッド
    fn verify_signature(&self, public_key: &str) -> bool {
        // 署名を検証するロジックをここに記述
        // この例では、SHA256を使用して署名を比較します
        let computed_signature = Sha256::digest(format!("{:?}", self).as_bytes()).to_vec();
        base64::encode(&computed_signature) == self.signature
    }
    
    // 新しいメソッドを追加
    fn generate_proof_of_history(&self) -> String {
        let mut hasher = Sha256::new();
        // トランザクションの情報をハッシュに追加
        hasher.update(format!("{:?}{:?}{:?}{:?}", self.sender, self.receiver, self.amount, self.timestamp).as_bytes());
        // ハッシュを生成して文字列として返す
        hex::encode(hasher.finalize())
    }

    // Transaction構造体の新しいインスタンスを作成するための関連関数
    fn new(
        sender: String,
        receiver: String,
        amount: f64,
        verifiable_credential: String,
        signature: String,  // String型で受け取る
        location: (f64, f64),
        subject: String,
        action_level: String,
        dimension: String,
        fluctuation: String,
        organism_name: String,
        details: String,
        goods_or_money: String,
        transaction_type: String,  // 新しく追加するフィールド
        sender_municipality: String, // 追加
        receiver_municipality: String, // 追加
        sender_continent: String,  // 新しく追加
        receiver_continent: String,  // 新しく追加
        status: String,  // 新しく追加
        created_at: DateTime<Utc>,  // 新しく追加
        sender_municipal_id: String,   // 追加
        receiver_municipal_id: String, // 追加
    ) -> Self {
        // 受け取ったbase64エンコードされた文字列をVec<u8>にデコード
        let signature_bytes = match base64::decode(&signature) {
            Ok(bytes) => bytes,
            Err(_) => vec![],  // デコードに失敗した場合、空のVecを返す
        };
        
        // base64文字列を再度エンコードする（必要に応じて処理）
        let signature_reencoded = base64::encode(&signature_bytes);

        // Transactionインスタンスを作成
        Transaction {
            sender,
            receiver,
            amount,
            verifiable_credential,
            signature: signature_reencoded,  // ここで再度base64エンコードした文字列を使う
            location: format!("{},{}", location.0, location.1),  // locationを文字列に変換
            timestamp: Utc::now(),  // 現在のUTC時間を取得
            proof_of_place: String::new(),
            transaction_id: uuid::Uuid::new_v4().to_string(),  // UUIDを生成して設定
            transaction_type,  // ここでは `transaction_type` を使用
            sender_municipality,  // ここで設定
            receiver_municipality,  // ここで設定
            sender_continent,  // 新しく追加
            receiver_continent,  // 新しく追加
            subject,
            action_level,
            dimension,
            fluctuation,
            organism_name,
            details,
            goods_or_money,
            status,  // 新しく追加
            created_at,  // 新しく追加
            sender_municipal_id,  // 追加
            receiver_municipal_id, // 追加
        }
    }
}

// インデックスページを追加
#[get("/")]
fn index() -> &'static str {
    "Welcome to the Municipal Chain!"
}

fn determine_continent_from_municipality(municipality: &str) -> Option<String> {
    println!("municipality: {}", municipality);  // デバッグ用

    // municipalities.jsonから読み込んだデータを表示
    let municipalities_data = load_municipalities_data().ok()?;  
    println!("Loaded municipalities data: {:?}", municipalities_data);

    // 'continent-city' 形式で大陸と市町村名を分離
    let parts: Vec<&str> = municipality.split('-').collect();
    if parts.len() != 2 {
        println!("Invalid municipality format: {}", municipality);
        return None;  // Noneを返してエラーとして処理する方が良いかもしれません
    }

    let continent = parts[0];  // 大陸名を取得
    let city_name = parts[1];  // 市町村名を取得

    println!("Continent: {}, City: {}", continent, city_name);  // デバッグ用

    // 該当する大陸の設定を取得
    if let Some(continent_config) = municipalities_data.get(continent) {
        if let Some(cities) = continent_config.get("cities").and_then(|v| v.as_array()) {
            for city in cities {
                if let Some(name) = city.get("name").and_then(|v| v.as_str()) {
                    if name == city_name {
                        if let Some(city_flask_port) = city.get("city_flask_port").and_then(|v| v.as_str()) {
                            println!("Found port for {}: {}", municipality, city_flask_port);
                            return Some(format!("http://127.0.0.1:{}", city_flask_port));
                        }
                    }
                }
            }
        }

        if let Some(flask_port) = continent_config.get("flask_port").and_then(|v| v.as_str()) {
            println!("Using continent default port: {}", flask_port);
            return Some(format!("http://127.0.0.1:{}", flask_port));
        }
    }

    if let Some(default_config) = municipalities_data.get("Default") {
        if let Some(flask_port) = default_config.get("flask_port").and_then(|v| v.as_str()) {
            println!("Using global default port: {}", flask_port);
            return Some(format!("http://127.0.0.1:{}", flask_port));
        }
    }

    None
}

async fn initialize_mongodb(continent: &str) -> Collection<Document> {
    let instance_type = "send_pending";  // ここは必要に応じて変更
    let mongo_uri = get_mongo_uri(instance_type, continent);
    
    let client_options = ClientOptions::parse(&mongo_uri).await.expect("MongoDB client options failed");
    let mongo_client = MongoClient::with_options(client_options).expect("MongoDB client failed");
    mongo_client.database("transactions_db").collection::<Document>("transactions")
}

async fn move_transaction_to_analytics(transaction: &Transaction) -> Result<(), Box<dyn std::error::Error>> {
    // MongoDB に接続
    let client_uri = "mongodb://localhost:27017/";
    let client_options = ClientOptions::parse(client_uri).await?;
    let client = Client::with_options(client_options)?;

    // データベースとコレクションを取得
    let original_db = client.database("original_database");
    let transactions_collection = original_db.collection::<Transaction>("transactions");

    let analytics_db = client.database("analytics");
    let analytics_collection = analytics_db.collection::<Transaction>("transactions");

    // トランザクションを `analytics` に挿入
    analytics_collection.insert_one(transaction, None).await?;

    // 元のコレクションから削除
    transactions_collection.delete_one(doc! { "transaction_id": &transaction.transaction_id }, None).await?;

    println!("Transaction {} moved to analytics database.", transaction.transaction_id);

    Ok(())
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct UpdateStatusRequest {
    transaction_id: String,
    new_status: String,
    municipal_id: String, // municipal_id を追加
}

async fn update_transaction_status(
    transaction_id: &str,
    municipal_id: &str,
    new_status: &str,
    collection: &Collection<Transaction>,
) -> Result<(), Box<dyn Error>> {
    // mongo_config.json を読み込む
    let mongo_config = load_mongo_config()?;
    
    // 大陸名を取得（municipal_id から大陸名を取得するロジックが必要です）
    // ここでは仮に municipal_id が大陸名であると仮定します
    let continent = municipal_id; // 必要に応じて修正してください

    // analytics_uri を取得
    let analytics_uri = mongo_config.analytics.get(continent)
        .or_else(|| mongo_config.analytics.get("Default"))
        .ok_or("Analytics URI not found in configuration")?;

    let filter = doc! {
        "transaction_id": transaction_id,
        "$or": [
            { "sender_municipal_id": municipal_id },
            { "receiver_municipal_id": municipal_id },
        ]
    };
    let update = doc! { "$set": { "status": new_status } };

    // ステータスを更新
    let result = collection.update_one(filter.clone(), update, None).await?;
    if result.matched_count == 0 {
        println!("Transaction {} not found.", transaction_id);
        return Err(Box::new(MongoError::from(mongodb::error::ErrorKind::WriteError(
            mongodb::error::WriteFailure::WriteError(mongodb::error::WriteError {
                code: 11000,
                code_name: "NotFound".to_string(),
                message: "Transaction not found".to_string(),
            }),
        ))));
    }
    println!("Transaction {} updated to status {}", transaction_id, new_status);

    // ステータスが "complete" になった場合、分析用データベースに移行
    if new_status == "complete" {
        // トランザクションを取得
        if let Some(transaction) = collection.find_one(filter.clone(), None).await? {
            // 分析用MongoDBに接続
            let analytics_client = mongodb::Client::with_uri_str(analytics_uri).await?;
            let analytics_db_name = analytics_uri.split('/').last().unwrap_or("analytics_db");
            let analytics_db = analytics_client.database(analytics_db_name);
            let analytics_collection = analytics_db.collection::<Transaction>("transactions");

            // トランザクションを分析用データベースに挿入
            analytics_collection.insert_one(transaction.clone(), None).await?;

            // 元のコレクションからトランザクションを削除
            collection.delete_one(filter, None).await?;

            println!("Transaction {} moved to analytics database.", transaction_id);
        } else {
            println!("Transaction {} not found during migration.", transaction_id);
            return Err(Box::new(MongoError::from(mongodb::error::ErrorKind::WriteError(
                mongodb::error::WriteFailure::WriteError(mongodb::error::WriteError {
                    code: 11000,
                    code_name: "NotFound".to_string(),
                    message: "Transaction not found during migration".to_string(),
                }),
            ))));
        }
    }

    Ok(())
}

#[post("/update_status", format = "json", data = "<update_request>")]
async fn update_status(
    update_request: Json<UpdateStatusRequest>,
    state: &State<Arc<Mutex<AppState>>>,
    client: &State<reqwest::Client>,
) -> Result<Status, Status> {
    let update_request = update_request.into_inner();
    let transaction_id = update_request.transaction_id;
    let new_status = update_request.new_status;
    let municipal_id = update_request.municipal_id;

    let state_guard = state.lock().unwrap(); // 非同期環境ではない場合は `lock().unwrap()` を使用
    let filter = doc! { "transaction_id": &transaction_id };
    let update = doc! { "$set": { "status": &new_status, "updated_at": BsonDateTime::from_chrono(Utc::now()) } };

    match state_guard.mongo_collection.update_one(filter.clone(), update, None).await {
        Ok(result) => {
            if result.matched_count == 0 {
                println!("Transaction {} not found.", transaction_id);
                return Err(Status::NotFound);
            }
            println!("Transaction {} updated to status {}", transaction_id, new_status);

            // ステータスが "complete" になった場合、分析用データベースに移行
            if new_status == "complete" {
                // mongo_config.json を読み込む
                let mongo_config = match load_mongo_config() {
                    Ok(config) => config,
                    Err(e) => {
                        println!("Failed to load mongo_config.json: {:?}", e);
                        return Err(Status::InternalServerError);
                    }
                };

                // 大陸名を取得（municipal_id から大陸名を取得するロジックが必要です）
                let continent = municipal_id.clone(); // 必要に応じて修正してください

                // analytics_uri を取得
                let analytics_uri = mongo_config.analytics.get(&continent)
                    .or_else(|| mongo_config.analytics.get("Default"))
                    .ok_or("Analytics URI not found in configuration")
                    .map_err(|e| {
                        println!("Analytics URI not found: {:?}", e);
                        Status::InternalServerError
                    })?;

                match state_guard.mongo_collection.find_one(filter.clone(), None).await {
                    Ok(Some(transaction_doc)) => {
                        // 分析用MongoDBに接続
                        let analytics_client = MongoClient::with_uri_str(analytics_uri).await.map_err(|e| {
                            println!("Failed to connect to analytics MongoDB: {:?}", e);
                            Status::InternalServerError
                        })?;
                        let analytics_db_name = analytics_uri.split('/').last().unwrap_or("analytics_db");
                        let analytics_db = analytics_client.database(analytics_db_name);
                        let analytics_collection = analytics_db.collection::<Document>("transactions");

                        // トランザクションを分析用データベースに挿入
                        match analytics_collection.insert_one(transaction_doc.clone(), None).await {
                            Ok(_) => {
                                println!("Transaction {} migrated to analytics DB.", transaction_id);
                                // 元のコレクションからトランザクションを削除
                                if let Err(e) = state_guard.mongo_collection.delete_one(filter.clone(), None).await {
                                    println!("Failed to delete transaction from operational DB: {:?}", e);
                                    return Err(Status::InternalServerError);
                                }
                                println!("Transaction {} removed from operational DB.", transaction_id);
                            },
                            Err(e) => {
                                println!("Failed to migrate transaction to analytics DB: {:?}", e);
                                return Err(Status::InternalServerError);
                            }
                        }
                    },
                    Ok(None) => {
                        println!("Transaction not found during migration: {}", transaction_id);
                        return Err(Status::NotFound);
                    },
                    Err(e) => {
                        println!("Error finding transaction during migration: {:?}", e);
                        return Err(Status::InternalServerError);
                    }
                }
            }

            Ok(Status::Ok)
        },
        Err(e) => {
            println!("Failed to update transaction: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[post("/transaction", format = "json", data = "<transaction>")]
async fn create_transaction(
    mut transaction: Json<Transaction>,
    client: &rocket::State<Client>,
    dpos: &rocket::State<Arc<Mutex<DPoS>>>,
    state: &rocket::State<Arc<Mutex<AppState>>>,
) -> Status {
    let mut transaction = transaction.into_inner();

    // `status`と`created_at`を設定
    transaction.status = "send_pending".to_string();
    transaction.created_at = Utc::now();

    // chrono::DateTime<Utc> を Bson::DateTime に変換
    let bson_timestamp = BsonDateTime::from_millis(transaction.timestamp.timestamp_millis());
    let bson_created_at = BsonDateTime::from_millis(transaction.created_at.timestamp_millis());

    // MongoDBにトランザクションを挿入
    let new_doc = doc! {
        "transaction_id": &transaction.transaction_id,
        "sender": &transaction.sender,
        "receiver": &transaction.receiver,
        "amount": &transaction.amount,
        "verifiable_credential": &transaction.verifiable_credential,
        "signature": &transaction.signature,
        "location": &transaction.location,
        "timestamp": bson_timestamp, // 変換したBson::DateTimeを使用
        "proof_of_place": &transaction.proof_of_place,
        "subject": &transaction.subject,
        "action_level": &transaction.action_level,
        "dimension": &transaction.dimension,
        "fluctuation": &transaction.fluctuation,
        "organism_name": &transaction.organism_name,
        "details": &transaction.details,
        "goods_or_money": &transaction.goods_or_money,
        "transaction_type": &transaction.transaction_type,
        "sender_municipality": &transaction.sender_municipality,  // 追加
        "receiver_municipality": &transaction.receiver_municipality,  // 追加
        "sender_continent": &transaction.sender_continent,  // 新しく追加
        "receiver_continent": &transaction.receiver_continent,  // 新しく追加
        "status": &transaction.status,
        "created_at": bson_created_at,
        "sender_municipal_id": &transaction.sender_municipal_id,  // 新しく追加
        "receiver_municipal_id": &transaction.receiver_municipal_id,  // 新しく追加
    };

    let app_state = state.lock().await;
    match app_state.mongo_collection.insert_one(new_doc, None).await {
        Ok(_) => println!("Transaction inserted successfully."),
        Err(err) => {
            println!("Failed to insert transaction: {:?}", err);
            return Status::InternalServerError;
        }
    }

    // ユーザーの評価データを更新
    update_user_evaluations(state.clone(), &transaction).await;

    // トランザクションを保留リストに追加
    {
        let mut pending_transactions_guard = app_state.pending_transactions.lock().await;
        pending_transactions_guard.insert(transaction.transaction_id.clone(), transaction.clone());
        println!("Transaction added to pending list: {:?}", transaction.transaction_id);
    }

    Status::Accepted

}

// バッチ処理用の関数を追加
// approve_and_send_transactions 関数
async fn approve_and_send_transactions(state: Arc<Mutex<AppState>>, client: Client) {
    loop {
        // バッチ間の待機時間（3秒）
        tokio::time::sleep(Duration::from_secs(3)).await;

        // 保留中のトランザクションを取得
        let transactions_to_process = {
            let state_guard = state.lock().await;
            let mut pending_guard = state_guard.pending_transactions.lock().await;

            // 300件のトランザクションを取得
            let txs: Vec<Transaction> = pending_guard.values().cloned().take(300).collect();

            // 取得したトランザクションを保留リストから削除
            for tx in &txs {
                pending_guard.remove(&tx.transaction_id);
            }

            txs
        };

        // 処理するトランザクションがない場合は次のループへ
        if transactions_to_process.is_empty() {
            continue;
        }

        // DPoSによるトランザクションの承認
        let approved_transactions: Vec<Transaction> = {
            let mut approved_txs = Vec::new();
            let state_guard = state.lock().await;
            let mut dpos_guard = state_guard.dpos.lock().await;

            for tx in transactions_to_process {
                // 送信者の市町村で代表者を選出
                dpos_guard.elect_representative(&tx.sender_municipality);

                // トランザクションを承認
                if dpos_guard.approve_transaction(&tx) {
                    approved_txs.push(tx);
                }
            }

            approved_txs
        };

        // 承認されたトランザクションがない場合は次のループへ
        if approved_transactions.is_empty() {
            println!("No transactions approved in this batch.");
            continue;
        }

        // ブロックを生成
        let new_block = {
            let state_guard = state.lock().await;
            let blockchain_guard = state_guard.blockchain.lock().await;

            let index = blockchain_guard.len() as u64;
            let prev_hash = if let Some(last_block) = blockchain_guard.last() {
                last_block.hash.clone()
            } else {
                "genesis".to_string()
            };

            let block_data = serde_json::to_string(&approved_transactions).unwrap();

            let mut block = Block {
                index,
                timestamp: Utc::now(),
                data: block_data,
                prev_hash,
                hash: "".to_string(), // 後で計算
                verifiable_credential: "credential".to_string(),
                signature: vec![],
            };

            // ブロックのハッシュを計算
            block.hash = calculate_block_hash(&block);

            block
        };

        // DPoSによるブロックの承認
        let block_approved = {
            let state_guard = state.lock().await;
            let mut dpos_guard = state_guard.dpos.lock().await;

            // ブロック承認のために代表者を選出（例として最初のトランザクションの市町村を使用）
            if let Some(first_tx) = approved_transactions.first() {
                dpos_guard.elect_representative(&first_tx.sender_municipality);
            }

            dpos_guard.approve_block(&new_block)
        };

        if block_approved {
            // ブロックチェーンに追加
            {
                let mut state_guard = state.lock().await;
                let mut blockchain_guard = state_guard.blockchain.lock().await;
                blockchain_guard.push(new_block.clone());
            }

            // MongoDBのトランザクションステータスを更新
            {
                let state_guard = state.lock().await;
                for tx in &approved_transactions {
                    let filter = doc! { "transaction_id": &tx.transaction_id };
                    let update = doc! { "$set": { "status": "approved", "block_id": &new_block.hash } };
                    if let Err(e) = state_guard.mongo_collection.update_one(filter, update, None).await {
                        println!("Failed to update transaction status for {}: {:?}", tx.transaction_id, e);
                    }
                }
            }

            // `continental_main_chain`にトランザクションを送信
            for tx in &approved_transactions {
                if let Err(e) = send_to_continental_chain(tx, &client).await {
                    println!("Error sending transaction {} to Continental Chain: {:?}", tx.transaction_id, e);
                }
            }
        } else {
            println!("Block approval failed by DPoS.");
            // ブロックが承認されなかった場合の処理（必要に応じて実装）
        }
    }
}

// ブロックのハッシュを計算する関数を追加
fn calculate_block_hash(block: &Block) -> String {
    let block_data = format!("{:?}{:?}{:?}{:?}", block.index, block.timestamp, block.data, block.prev_hash);
    let mut hasher = Sha256::new();
    hasher.update(block_data.as_bytes());
    hex::encode(hasher.finalize())
}

#[post("/add_block", format = "json", data = "<block>")]
async fn add_block(block: Json<Block>, chain: &rocket::State<Blockchain>, client: &rocket::State<Client>) -> Status {
    let mut chain = chain.lock().await;
    let block_clone = block.clone();  
    chain.push(block.into_inner());

    let global_chain_url = "http://global_main_chain:1999/add_block";
    let res = client.post(global_chain_url)
                    .json(&*block_clone)  
                    .send()
                    .await;

    match res {
        Ok(_) => Status::Accepted,
        Err(_) => Status::InternalServerError,
    }
}

async fn schedule_representative_selection(state: Arc<Mutex<AppState>>) {
    let mut interval = tokio::time::interval(Duration::days(1)); // 毎日チェック
    loop {
        interval.tick().await;

        let now = Utc::now();
        let day = now.day();
        let month = now.month();

        // 代表者選出の日付チェック（例：7月1日、10月1日、1月1日、4月1日）
        if day == 1 && (month == 7 || month == 10 || month == 1 || month == 4) {
            // 前の評価期間の開始と終了を計算
            let evaluation_end = now - Duration::days(1); // 前日まで
            let evaluation_start = evaluation_end - Duration::days(90); // 90日前

            // 評価期間のデータに基づいて代表者を選出
            select_representatives(state.clone(), evaluation_start, evaluation_end).await;
        }
    }
}

async fn select_representatives(state: Arc<Mutex<AppState>>, start_date: DateTime<Utc>, end_date: DateTime<Utc>) {
    let state_guard = state.lock().await;

    // ユーザー評価データを取得
    let user_evaluations = state_guard.user_evaluations.lock().await;

    // 市町村ごとにユーザーをグループ化してランキング
    let mut municipal_rankings: HashMap<String, Vec<(String, f64)>> = HashMap::new();

    for (user_id, eval) in user_evaluations.iter() {
        // ユーザーの市町村を取得（仮定: get_user_municipality 関数で取得）
        let municipality = get_user_municipality(user_id).await;

        let total_score = eval.total_love_token_usage + eval.value_from_tokens_received + eval.contribution_by_token_usage;

        municipal_rankings
            .entry(municipality)
            .or_insert(Vec::new())
            .push((user_id.clone(), total_score));
    }

    // 市町村ごとに上位5名を選出
    let mut new_representatives: HashMap<String, Vec<Representative>> = HashMap::new();

    for (municipality, mut rankings) in municipal_rankings {
        // スコアの高い順にソート
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 上位5名を選出
        let top_5 = rankings.into_iter().take(5);

        let representatives = top_5.map(|(user_id, _)| Representative {
            user_id,
            start_date: end_date + Duration::days(90), // 任期開始は3ヶ月後
            end_date: end_date + Duration::days(90 * 2), // 任期はその3ヶ月後まで
        }).collect::<Vec<_>>();

        new_representatives.insert(municipality.clone(), representatives);
    }

    // 代表者リストを更新
    let mut municipal_representatives = state_guard.municipal_representatives.lock().await;
    *municipal_representatives = new_representatives.clone();

    // メッセージの送信（省略: send_message_to_user 関数を仮定）
    for (municipality, reps) in new_representatives.iter() {
        for rep in reps {
            // 新しい代表者へのメッセージ
            send_message_to_user(&rep.user_id, "10月1日～12月31日までの期間、代表よろしくお願いします").await;
        }
    }

    // 以前の代表者へのメッセージ（過去の代表者リストを保持している場合）
    // ここでは簡略化のため省略
}

async fn send_message_to_user(user_id: &str, message: &str) {
    // ユーザーにメッセージを送信する処理を実装
    // ここでは詳細を省略
    println!("Message to {}: {}", user_id, message);
}

async fn calculate_contribution(transaction: &Transaction) -> f64 {
    // トランザクションの内容に基づいて貢献度を計算するロジックを実装
    // ここでは仮に amount の値をそのまま返す
    transaction.amount
}

async fn approve_block(transaction_id: &str) -> Result<(), Box<dyn Error>> {
    // ブロック承認ロジック（実装は省略）
    // 承認が成功したらステータスを "send_complete" に更新

    // MongoDB設定から `update_status_url` を取得
    let mongodb_config = load_mongodb_config().expect("Failed to load MongoDB config");

    // `update_status_url` が存在するか確認し、なければエラーを返す
    let update_status_url = mongodb_config.get("update_status_url")
        .and_then(|url| url.as_str())
        .unwrap_or("http://localhost:10600/update_status");

    let client = Client::new();
    let update_status_req = serde_json::json!({
        "transaction_id": transaction_id,
        "new_status": "send_complete"
    });

    // 動的に取得した `update_status_url` に対してリクエストを送信
    let res = client.post(update_status_url)
        .json(&update_status_req)
        .send()
        .await?;

    if res.status().is_success() {
        println!("Transaction {} status updated to 'send_complete'", transaction_id);
        Ok(())
    } else {
        println!("Failed to update transaction {} status: {:?}", transaction_id, res.text().await);
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to update transaction status",
        )))
    }
}

// トランザクションの作成や受信時に、ユーザーの評価データを更新する
async fn update_user_evaluations(state: Arc<Mutex<AppState>>, transaction: &Transaction) {
    let mut state_guard = state.lock().await;
    let mut user_evaluations = state_guard.user_evaluations.lock().await;

    // 送信者の評価データを更新
    let sender_eval = user_evaluations.entry(transaction.sender.clone()).or_insert(EvaluationItems {
        total_love_token_usage: 0.0,
        value_from_tokens_received: 0.0,
        contribution_by_token_usage: 0.0,
    });
    sender_eval.total_love_token_usage += transaction.amount;
    sender_eval.contribution_by_token_usage += calculate_contribution(transaction).await;

    // 受信者の評価データを更新
    let receiver_eval = user_evaluations.entry(transaction.receiver.clone()).or_insert(EvaluationItems {
        total_love_token_usage: 0.0,
        value_from_tokens_received: 0.0,
        contribution_by_token_usage: 0.0,
    });
    receiver_eval.value_from_tokens_received += transaction.amount;
}

async fn get_transactions_by_municipal_id(
    municipal_id: String,
    collection: &Collection<Transaction>,
) -> Result<Vec<Transaction>, mongodb::error::Error> {
    let filter = doc! {
        "$or": [
            { "sender_municipal_id": &municipal_id },
            { "receiver_municipal_id": &municipal_id },
        ]
    };
    let mut cursor = collection.find(filter, None).await?;
    let mut transactions = Vec::new();

    while let Some(transaction) = cursor.try_next().await? {
        transactions.push(transaction);
    }

    Ok(transactions)
}

// 受信処理
#[post("/receive_transaction_by_id", format = "json", data = "<transaction_id>")]
async fn receive_transaction_by_id(transaction_id: Json<String>, state: &State<PendingTransactions>) -> Status {
    let transaction_id = transaction_id.into_inner();
    let mut pending_transactions = state.lock().await;

    if let Some(transaction) = pending_transactions.remove(&transaction_id) {
        // トランザクションのステータスを 'receive_pending' に更新
        let result = state_guard.mongo_collection.update_one(
            doc! { "transaction_id": &transaction.transaction_id },
            doc! { "$set": { "status": "receive_pending", "updated_at": BsonDateTime::from_chrono(Utc::now()) } },
            None
        ).await;

        if result.matched_count == 0 {
            println!("Failed to update transaction status to receive_pending for transaction: {}", transaction.transaction_id);
            return Status::InternalServerError;
        }

        // Continental Main Chainへ通知
        let receiver_continent_url = determine_continent_from_municipality(&transaction.receiver_municipality);

        if let Some(url) = receiver_continent_url {
            let res = Client::new().post(&format!("{}/complete_transaction", url)).json(&transaction).send().await;

            if res.is_ok() {
                Status::Accepted
            } else {
                Status::InternalServerError
            }
        } else {
            Status::InternalServerError
        }
    } else {
        Status::NotFound
    }
}

// DAppsからのトランザクション受信処理を追加
#[post("/receive_transaction", format = "json", data = "<transaction>")]
async fn receive_transaction<'r>(
    transaction: Result<Json<serde_json::Value>, rocket::serde::json::Error<'_>>, 
    dpos: &rocket::State<Arc<Mutex<DPoS>>>,  
    state: &rocket::State<Arc<Mutex<AppState>>>,  
    client: &rocket::State<Client>,  
) -> Result<Status, Status> {
    println!("Receiving transaction...");
    let mut dpos_guard = dpos.lock().await;  

    if let Ok(json_value) = transaction {
        // 必要なフィールドを取得
        if let (
            Some(transaction_id),
            Some(sender),
            Some(receiver),
            Some(amount),
            Some(timestamp_str),
            Some(tx_type),
            Some(sender_municipality),
            Some(receiver_municipality),
            Some(sender_continent),
            Some(receiver_continent),
            Some(status),
            Some(created_at_str),
            Some(sender_municipal_id),
            Some(receiver_municipal_id),
        ) = (
            json_value.get("transaction_id").and_then(|t| t.as_str()),
            json_value.get("sender").and_then(|s| s.as_str()),
            json_value.get("receiver").and_then(|r| r.as_str()),
            json_value.get("amount").and_then(|a| a.as_f64()),
            json_value.get("timestamp").and_then(|ts| ts.as_str()),
            json_value.get("transaction_type").and_then(|tt| tt.as_str()),
            json_value.get("sender_municipality").and_then(|sm| sm.as_str()),
            json_value.get("receiver_municipality").and_then(|rm| rm.as_str()),
            json_value.get("sender_continent").and_then(|sc| sc.as_str()),
            json_value.get("receiver_continent").and_then(|rc| rc.as_str()),
            json_value.get("status").and_then(|st| st.as_str()),
            json_value.get("created_at").and_then(|ca| ca.as_str()),
            json_value.get("sender_municipal_id").and_then(|ct| ct.as_str()),
            json_value.get("receiver_municipal_id").and_then(|cs| cs.as_str()),
        ) {
            // Parse the `timestamp` and `created_at` strings into `DateTime<Utc>`
            match (parse_timestamp(timestamp_str), parse_timestamp(created_at_str)) {
                (Ok(parsed_timestamp), Ok(parsed_created_at)) => {
                    // トランザクションを生成
                    let transaction = Transaction {
                        transaction_id: transaction_id.to_string(),
                        sender: sender.to_string(),
                        receiver: receiver.to_string(),
                        amount,
                        verifiable_credential: json_value.get("verifiable_credential").and_then(|vc| vc.as_str()).unwrap_or_default().to_string(),
                        signature: json_value.get("signature").and_then(|sig| sig.as_str()).unwrap_or_default().to_string(),
                        timestamp: parsed_timestamp,
                        subject: json_value.get("subject").and_then(|sub| sub.as_str()).unwrap_or_default().to_string(),
                        action_level: json_value.get("action_level").and_then(|al| al.as_str()).unwrap_or_default().to_string(),
                        dimension: json_value.get("dimension").and_then(|dim| dim.as_str()).unwrap_or_default().to_string(),
                        fluctuation: json_value.get("fluctuation").and_then(|fluc| fluc.as_str()).unwrap_or_default().to_string(),
                        organism_name: json_value.get("organism_name").and_then(|on| on.as_str()).unwrap_or_default().to_string(),
                        details: json_value.get("details").and_then(|d| d.as_str()).unwrap_or_default().to_string(),
                        goods_or_money: json_value.get("goods_or_money").and_then(|gom| gom.as_str()).unwrap_or_default().to_string(),
                        transaction_type: tx_type.to_string(),
                        sender_municipality: sender_municipality.to_string(),
                        receiver_municipality: receiver_municipality.to_string(),
                        sender_continent: sender_continent.to_string(),
                        receiver_continent: receiver_continent.to_string(),
                        status: status.to_string(),
                        created_at: parsed_created_at,
                        sender_municipal_id: sender_municipal_id.to_string(),
                        receiver_municipal_id: receiver_municipal_id.to_string(),
                    };

                    println!("DPoS transaction approval in progress...");

                    if tx_type == "send" {
                        dpos_guard.elect_representative(&transaction.sender_municipality);
                        if dpos_guard.approve_transaction(&transaction) {
                            let state_guard = state.lock().await;  // Lock `state` first
                            let mut pending_transactions_guard = state_guard.pending_transactions.lock().await;  // Now lock `pending_transactions`

                            pending_transactions_guard.insert(transaction.transaction_id.clone(), transaction.clone());

                            // トランザクションをデータベースに保存
                            match state_guard.mongo_collection.insert_one(transaction.clone(), None).await {
                                Ok(_) => {
                                    // Global Chain にトランザクションを転送
                                    let global_chain_url = "http://global_main_chain:1999/receive_transaction";
                                    let res = client.post(global_chain_url)
                                                    .json(&transaction)
                                                    .send()
                                                    .await;

                                    if res.is_ok() {
                                        let update_status_req = serde_json::json!({
                                            "transaction_id": transaction.transaction_id,
                                            "new_status": "send_complete",
                                            "sender_municipal_id": transaction.sender_municipal_id,
                                            "continent": transaction.sender_continent
                                        });
                                        let update_res = client.post("http://localhost:1060/update_status")
                                            .json(&update_status_req)
                                            .send()
                                            .await;

                                        if update_res.is_ok() && update_res.unwrap().status().is_success() {
                                            println!("Transaction approved and status updated to 'send_complete'.");
                                            return Ok(Status::Accepted);
                                        } else {
                                            println!("Failed to update transaction status to 'send_complete'.");
                                            return Ok(Status::InternalServerError);
                                        }
                                    } else {
                                        println!("Failed to forward transaction to Global Chain.");
                                        return Ok(Status::InternalServerError);
                                    }
                                }
                                Err(e) => {
                                    println!("Failed to save transaction to database: {}", e);
                                    return Ok(Status::InternalServerError);
                                }
                            }
                        } else {
                            println!("Transaction approval failed by sender's representative.");
                            return Ok(Status::Forbidden);
                        }
                    } else if tx_type == "receive" {
                        dpos_guard.elect_representative(&transaction.receiver_municipality);
                        if dpos_guard.approve_transaction(&transaction) {
                            // トランザクションをデータベースに保存
                            let state_guard = state.lock().await;
                            match state_guard.mongo_collection.insert_one(transaction.clone(), None).await {
                                Ok(_) => {
                                    let update_status_req = serde_json::json!({
                                        "transaction_id": transaction.transaction_id,
                                        "new_status": "complete",
                                        "receiver_municipal_id": transaction.receiver_municipal_id,
                                        "continent": transaction.receiver_continent
                                    });
                                    let update_res = client.post("http://localhost:1060/update_status")
                                        .json(&update_status_req)
                                        .send()
                                        .await;

                                    if update_res.is_ok() && update_res.unwrap().status().is_success() {
                                        println!("Transaction approved and status updated to 'complete'.");
                                        return Ok(Status::Accepted);
                                    } else {
                                        println!("Failed to update transaction status to 'complete'.");
                                        return Ok(Status::InternalServerError);
                                    }
                                }
                                Err(e) => {
                                    println!("Failed to save transaction to database: {}", e);
                                    return Ok(Status::InternalServerError);
                                }
                            }
                        } else {
                            println!("Transaction approval failed by receiver's representative.");
                            return Ok(Status::Forbidden);
                        }
                    } else {
                        println!("Invalid transaction type.");
                        return Err(Status::BadRequest);
                    }
                }
                _ => {
                    println!("Failed to parse timestamp or created_at");
                    return Err(Status::UnprocessableEntity);
                }
            }
        } else {
            println!("Missing required fields in JSON payload");
            return Err(Status::BadRequest);
        }
    } else {
        println!("Failed to parse JSON payload");
        return Err(Status::BadRequest);
    }
}

// トランザクションに署名を追加する関数
fn sign_transaction(transaction: &mut Transaction, private_key: &NtruSign) {
    let message = format!("{:?}{:?}{:?}{:?}", transaction.sender, transaction.receiver, transaction.amount, transaction.timestamp);
    let signature = private_key.sign(&message.as_bytes());
    transaction.signature = base64::encode(signature);
}

// 署名を検証する関数
fn verify_transaction_signature(transaction: &Transaction, public_key: &NtruSign) -> bool {
    let message = format!("{:?}{:?}{:?}{:?}", transaction.sender, transaction.receiver, transaction.amount, transaction.timestamp);
    let signature_bytes = base64::decode(&transaction.signature).unwrap();
    public_key.verify(&message.as_bytes(), &signature_bytes)
}

// トランザクションデータを暗号化する関数
fn encrypt_transaction(transaction: &Transaction, public_key: &NtruEncrypt) -> Vec<u8> {
    let serialized_transaction = serde_json::to_string(transaction).unwrap();
    public_key.encrypt(&serialized_transaction.as_bytes())
}

// トランザクションデータを復号化する関数
fn decrypt_transaction(encrypted_data: &[u8], private_key: &NtruEncrypt) -> Transaction {
    let decrypted_data = private_key.decrypt(encrypted_data);
    let decrypted_string = String::from_utf8(decrypted_data).unwrap();
    serde_json::from_str(&decrypted_string).unwrap()
}

fn validate_transaction(transaction: &Transaction) -> Result<(), String> {
    // 例: sender_municipal_id が正しい形式であるかを確認
    if transaction.sender_municipal_id.is_empty() {
        return Err("sender_municipal_id is missing".to_string());
    }
    if transaction.receiver_municipal_id.is_empty() {
        return Err("receiver_municipal_id is missing".to_string());
    }
    // 他の検証ロジック
    Ok(())
}

#[get("/pending_transactions")]
async fn get_pending_transactions(state: &State<PendingTransactions>) -> Json<Vec<Transaction>> {
    let pending_transactions = state.lock().await;
    Json(pending_transactions.values().cloned().collect())
}

fn is_local_transaction(transaction: &Transaction) -> bool {
    // トランザクションがローカルかどうかを判定
    // ここにローカル判定ロジックを実装
    true // 仮実装としてtrueを返す
}

fn process_local_transaction(transaction: Transaction) {
    // ローカルトランザクションの処理
    println!("Processing local transaction: {:?}", transaction);
}

// トランザクションをcontinental_chainに送信する関数
async fn send_to_continental_chain(transaction: &Transaction, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let continent_url = determine_continent_from_municipality(&transaction.sender_municipality);

    if let Some(url) = continent_url {
        let res = client.post(&format!("{}/receive_transaction", url))
                        .json(transaction)
                        .send()
                        .await?;

        if res.status().is_success() {
            println!("Transaction successfully sent to Continental Chain");
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to send transaction to Continental Chain",
            )))
        }
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid continent URL",
        )))
    }
}

async fn get_data_from_mongodb(
    mongo_collection: &Collection<Document> // BSON ドキュメント型のコレクション
) -> Vec<Value> {
    let filter = doc! {}; // フィルタなしですべてのドキュメントを取得
    let mut cursor = mongo_collection.find(filter, None).await.expect("Find failed");

    let mut json_results = Vec::new();

    while let Some(result) = cursor.next().await {
        match result {
            Ok(bson_doc) => {
                // BSON ドキュメントを rocket::serde::json::Value に変換
                let json_value = bson_to_json(bson_doc);
                json_results.push(json_value);
            }
            Err(e) => println!("Error reading document: {:?}", e),
        }
    }

    json_results
}

fn bson_to_json(bson_doc: Document) -> Value {
    // BSON ドキュメントを rocket::serde::json::Value に変換
    rocket::serde::json::to_value(bson_doc).unwrap_or(Value::Null)
}

#[post("/gossip_transactions", format = "json", data = "<gossip_data>")]
async fn gossip_transactions_handler(
    gossip_data: Json<GossipRequest>,
    state: &State<Arc<Mutex<AppState>>>,
) -> Status {
    let state = state.lock().await;
    let mut pending_transactions = state.pending_transactions.lock().await;

    for (tx_id, transaction) in &gossip_data.transactions {
        pending_transactions.insert(tx_id.clone(), transaction.clone());
    }

    println!("Gossip transactions updated.");
    Status::Ok
}

// Gossipプロトコルを使って他の大陸チェーンとブロックを同期
async fn gossip_blocks_with_chains(state: Arc<Mutex<AppState>>, chain_urls: Vec<String>) {
    let state_guard = state.lock().await;
    let blockchain_guard = state_guard.blockchain.lock().await;

    // 最新のブロックをGossipデータとして構築
    if let Some(latest_block) = blockchain_guard.last().cloned() {
        for chain_url in chain_urls {
            let client = Client::new();
            let res = client.post(&format!("{}/gossip_blocks", chain_url))
                .json(&latest_block)
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Block gossip successfully processed by {}", chain_url);
                    } else {
                        let err_msg: String = response.text().await.unwrap_or("Unknown error".into());
                        println!("Failed to gossip block: {}", err_msg);
                    }
                },
                Err(err) => {
                    println!("Failed to reach chain for gossiping block: {:?}", err);
                },
            }
        }
    } else {
        println!("No block to gossip.");
    }
}

#[post("/gossip_blocks", format = "json", data = "<gossip_data>")]
async fn gossip_blocks_handler(
    gossip_data: Json<GossipBlockRequest>, 
    state: &State<Arc<Mutex<AppState>>>
) -> Status {
    let gossip_data = gossip_data.into_inner();
    
    // まずAppState全体のロックを取得
    let app_state = state.lock().await;
    
    // その後にブロックチェーンのロックを取得
    let mut blockchain = app_state.blockchain.lock().await;

    // 受け取ったブロックをブロックチェーンに追加
    for block in gossip_data.blocks {
        blockchain.push(block);
    }

    println!("Gossip blocks received and added to blockchain.");
    Status::Ok
}

pub mod immudb_proto {
    tonic::include_proto!("immudb");
}

use immudb_proto::immu_service_client::ImmuServiceClient;
use tonic::transport::Channel;

async fn connect_to_immudb(host: &str, port: u16) -> Result<ImmuServiceClient<Channel>, Box<dyn std::error::Error>> {
    let address = format!("http://{}:{}", host, port);
    let client = ImmuServiceClient::connect(address).await?;
    Ok(client)
}

async fn save_block_to_immudb(client: &mut ImmuServiceClient<Channel>, block: &Block) -> Result<(), Box<dyn std::error::Error>> {
    let block_json = serde_json::to_string(block)?;
    let block_key = format!("block_{}", block.index);

    let kv = KeyValue {
        key: block_key.into_bytes(),
        value: block_json.into_bytes(),
    };

    let request = tonic::Request::new(SetRequest {
        kvs: vec![kv],
    });

    client.set(request).await?;

    println!("Block {} saved to immudb.", block.index);

    Ok(())
}

async fn get_block_from_immudb(client: &mut ImmuServiceClient<Channel>, block_index: u64) -> Result<Block, Box<dyn std::error::Error>> {
    let block_key = format!("block_{}", block_index);

    let request = tonic::Request::new(GetRequest {
        key: block_key.into_bytes(),
        at_tx: 0,
        since_tx: 0,
        no_wait: false,
    });

    let response = client.get(request).await?;
    let value = response.into_inner().value;

    let block_json = String::from_utf8(value)?;
    let block: Block = serde_json::from_str(&block_json)?;

    Ok(block)
}

async fn clean_expired_send_pending_transactions(state: Arc<Mutex<AppState>>) {
    loop {
        // 6ヶ月前の日時を計算
        let expiration_threshold = Utc::now() - chrono::Duration::days(6 * 30);

        let state_guard = state.lock().await;
        let filter = doc! {
            "status": "send_pending",
            "created_at": { "$lt": BsonDateTime::from_millis(expiration_threshold.timestamp_millis()) }
        };

        match state_guard.mongo_collection.delete_many(filter, None).await {
            Ok(result) => {
                println!("Deleted {} expired send_pending transactions", result.deleted_count);
            },
            Err(e) => {
                println!("Error during cleanup: {:?}", e);
            }
        }

        // 24時間待機
        tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
    }
}

async fn create_indexes(collection: &Collection<Transaction>) -> Result<(), mongodb::error::Error> {
    // 既存のインデックス作成コードに追加
    collection.create_index(
        IndexModel::builder()
            .keys(doc! { "sender_municipal_id": 1 })
            .build(),
        None,
    ).await?;
    
    collection.create_index(
        IndexModel::builder()
            .keys(doc! { "receiver_municipal_id": 1 })
            .build(),
        None,
    ).await?;

    Ok(())
}

// 分析用データベースのインデックス作成関数を追加
async fn create_analytics_indexes(analytics_collection: &Collection<Document>) {
    let status_index = IndexModel::builder()
        .keys(doc! { "status": 1 })
        .options(None)
        .build();
    
    let created_at_index = IndexModel::builder()
        .keys(doc! { "created_at": 1 })
        .options(None)
        .build();

    analytics_collection.create_index(status_index, None)
        .await
        .expect("Failed to create index on status in analytics DB");

    analytics_collection.create_index(created_at_index, None)
        .await
        .expect("Failed to create index on created_at in analytics DB");

    println!("Indexes created on 'status' and 'created_at' fields in analytics database.");
}

// Rocketでのメイン関数
// Rocketでのメイン関数
#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    immudb_proto_function();

    // 大陸情報に基づいてポートとMongoDB URIを設定
    let args: Vec<String> = env::args().collect();
    let continent_city = if args.len() > 1 { &args[1] } else { "Default-Defaultcity" };

    // 設定取得
    let (port, mongo_uri) = get_city_ports_and_uri(continent_city).unwrap_or((1034, "mongodb://localhost:10034".to_string()));

    // municipalities.jsonファイルをロード
    let municipalities_data = load_municipalities_data().expect("Failed to load municipalities data");

    // **ここで mongo_db_name を宣言して定義**
    let mongo_db_name = format!("{}_municipal_chain_db", continent_city.to_lowercase().replace("-", "_"));

    // MongoDBクライアントとコレクションの初期化
    let client_options = ClientOptions::parse(&mongo_uri).await.expect("MongoDB client options failed");
    let mongo_client = MongoClient::with_options(client_options).expect("MongoDB client failed");

    // **ここで宣言された mongo_db_name を利用してコレクションを設定**
    let mongo_collection = mongo_client.database(&mongo_db_name).collection::<Document>("pending_transactions");

    // BlockchainとDPoSの初期化
    let blockchain = Arc::new(Mutex::new(Vec::<Block>::new()));
    let dpos = Arc::new(Mutex::new(DPoS::new(&municipalities_data)));

    // mongo_config.json を読み込む
    let mongo_config = load_mongo_config().expect("Failed to load mongo_config.json");

    // 大陸名を取得（continent_city は "Continent-City" の形式であると仮定）
    let continent = continent_city.split('-').next().unwrap_or("Default");

    // analytics_uri を取得
    let analytics_uri = mongo_config.analytics.get(continent)
        .or_else(|| mongo_config.analytics.get("Default"))
        .expect("Analytics URI not found in configuration")
        .clone();

    // 分析用MongoDBの初期化
    let analytics_client = MongoClient::with_uri_str(&analytics_uri).await.expect("Failed to connect to analytics MongoDB");

    // analytics_db_name を analytics_uri から取得
    let analytics_db_name = {
        let uri_parts: Vec<&str> = analytics_uri.split('/').collect();
        uri_parts.last().unwrap_or(&"analytics_db")
    };

    let analytics_collection = analytics_client.database(analytics_db_name).collection::<Document>("shared_transactions");

    // AppStateの作成
    let state = Arc::new(Mutex::new(AppState {
        pending_transactions: Arc::new(Mutex::new(HashMap::new())),
        blockchain: Arc::clone(&blockchain),
        mongo_collection: mongo_collection.clone(),
        other_continental_chains: Arc::new(Mutex::new(get_other_continental_chains(&municipalities_data, continent_city))),
        dpos: Arc::clone(&dpos),  // `dpos`をクローンして参照カウントを増やす
        mongo_config: mongo_config.clone(), // mongo_config を追加
        user_evaluations: Arc::new(Mutex::new(HashMap::new())),
        municipal_representatives: Arc::new(Mutex::new(HashMap::new())),
    }));

    // 代表者選出タスクを起動
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        schedule_representative_selection(state_clone).await;
    });

    // インデックスを作成
    {
        let state_guard = state.lock().await;
        state_guard.create_indexes().await;
    }

    // 分析用データベースのインデックスを作成
    create_analytics_indexes(&analytics_collection).await;

    // 期限付き削除タスクを起動
    let state_clone = Arc::clone(&state);
    task::spawn(async move {
        clean_expired_send_pending_transactions(state_clone).await;
    });

    // reqwest::Client の初期化
    let client = Client::new();

    // `approve_and_send_transactions` タスクを開始
    let state_clone2 = Arc::clone(&state);
    let client_clone2 = client.clone();
    task::spawn(async move {
        approve_and_send_transactions(state_clone2, client_clone2).await;
    });

    // Gossipプロトコルを使用して定期的に他のチェーンとブロックを共有するタスクを開始
    let state_clone = Arc::clone(&state);
    task::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;

            let gossip_urls = {
                let state_guard = state_clone.lock().await;
                let other_chains_guard = state_guard.other_continental_chains.lock().await;
                other_chains_guard.clone()
            };
            gossip_blocks_with_chains(state_clone.clone(), gossip_urls).await;
        }
    });

    // Rocket のインスタンスを作成
    rocket::build()
        .configure(Config {
            port,  // 動的に取得したポートを使用
            ..Config::default()
        })
        .manage(client)
        .manage(dpos.clone()) 
        .manage(state.clone()) 
        .mount("/", routes![
            index, 
            create_transaction, 
            receive_transaction, 
            gossip_transactions_handler, 
            gossip_blocks_handler,
            update_status // 必要に応じて追加
        ])
        .attach(AdHoc::on_liftoff("Liftoff message", |_| Box::pin(async move {
            println!("Rocket is launching...");
        })))
        .launch()
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::{bson::doc, bson::DateTime as BsonDateTime, Client as MongoClient};
    use chrono::Utc;
    use base64;
    use tokio;

    #[test]
    fn test_transaction_functions() {
        let signature_bytes = vec![];  // 空の署名データ
        let signature = base64::encode(signature_bytes);  // Vec<u8>をbase64エンコードしてStringに変換

        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            "VerifiableCredential".to_string(),
            signature,  // String型の署名データを渡す
            (35.0, 139.0),
            "subject".to_string(),
            "action_level".to_string(),
            "dimension".to_string(),
            "fluctuation".to_string(),
            "organism_name".to_string(),
            "details".to_string(),
            "goods_or_money".to_string(),
            "send".to_string(),  // transaction_type を設定
            "sender_municipality".to_string(),  // sender_municipality を設定
            "receiver_municipality".to_string(),  // receiver_municipality を設定
            "Asia".to_string(),  // sender_continent を設定
            "Europe".to_string(),  // receiver_continent を設定
            "pending".to_string(),  // ステータスを "pending" に設定
            chrono::Utc::now(), // corrected: created_at に現在の日時を渡す
            "sender_municipal_id".to_string(),
            "receiver_municipal_id".to_string(),
        );

        // テストとして関数を呼び出す
        let proof_of_history = transaction.generate_proof_of_history();
        assert!(!proof_of_history.is_empty());

        let signature_valid = transaction.verify_signature("public_key");
        assert!(!signature_valid);  // 現在のデータでは正しい署名がないためfalse
    }

    #[tokio::test]
    async fn test_update_status() {
        // テスト用にMongoDBのクライアントとコレクションを設定
        let mongo_uri = "mongodb://localhost:27017";  // テスト用MongoDBのURIに変更
        let client = MongoClient::with_uri_str(mongo_uri).await.unwrap();
        let db = client.database("test_transactions_db");
        let collection = db.collection::<Document>("transactions");

        // テスト用トランザクションを挿入
        let test_transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            "VerifiableCredential".to_string(),
            "dummy_signature".to_string(),
            (35.0, 139.0),
            "subject".to_string(),
            "action_level".to_string(),
            "dimension".to_string(),
            "fluctuation".to_string(),
            "organism_name".to_string(),
            "details".to_string(),
            "goods_or_money".to_string(),
            "send".to_string(),
            "Asia-Tokyo".to_string(),
            "Europe-London".to_string(),
            "Asia".to_string(),
            "Europe".to_string(),
            "pending".to_string(), // ステータスを "pending" に設定
            chrono::Utc::now(),         // created_at の日時
            "sender_municipal_id".to_string(),
            "receiver_municipal_id".to_string(),
        );

        let bson_created_at = BsonDateTime::from_millis(test_transaction.created_at.timestamp_millis());

        let new_doc = doc! {
            "transaction_id": &test_transaction.transaction_id,
            "sender": &test_transaction.sender,
            "receiver": &test_transaction.receiver,
            "amount": &test_transaction.amount,
            "verifiable_credential": &test_transaction.verifiable_credential,
            "signature": &test_transaction.signature,
            "location": {
                "latitude": test_transaction.location.0,
                "longitude": test_transaction.location.1,
            },
            "timestamp": BsonDateTime::from_millis(test_transaction.timestamp.timestamp_millis()),
            "proof_of_place": &test_transaction.proof_of_place,
            "subject": &test_transaction.subject,
            "action_level": &test_transaction.action_level,
            "dimension": &test_transaction.dimension,
            "fluctuation": &test_transaction.fluctuation,
            "organism_name": &test_transaction.organism_name,
            "details": &test_transaction.details,
            "goods_or_money": &test_transaction.goods_or_money,
            "transaction_type": &test_transaction.transaction_type,
            "sender_municipality": &test_transaction.sender_municipality,
            "receiver_municipality": &test_transaction.receiver_municipality,
            "sender_continent": &test_transaction.sender_continent,
            "receiver_continent": &test_transaction.receiver_continent,
            "status": &test_transaction.status,
            "created_at": bson_created_at,
            "sender_municipal_id": &test_transaction.sender_municipal_id,
            "receiver_municipal_id": &test_transaction.receiver_municipal_id,
        };

        collection.insert_one(new_doc.clone(), None).await.unwrap();

        // ステータスを "complete" に更新
        let update_status_req = UpdateStatusRequest {
            transaction_id: test_transaction.transaction_id.clone(),
            new_status: "complete".to_string(),
            municipal_id: "Asia-Tokyo".to_string(), // municipal_id を追加
        };

        // `update_status` 関数を直接呼び出すための準備
        let state = Arc::new(Mutex::new(AppState {
            mongo_collection: collection.clone(),
            mongo_config: load_mongo_config().expect("Failed to load mongo_config.json"),
            // 他の必要なフィールドを初期化
            pending_transactions: Arc::new(Mutex::new(HashMap::new())),
            blockchain: Arc::new(Mutex::new(Vec::new())),
            other_continental_chains: Arc::new(Mutex::new(Vec::new())),
            dpos: Arc::new(Mutex::new(DPoS::new(&HashMap::new()))),
        }));

        let client = reqwest::Client::new();

        // `update_status` 関数を呼び出し
        let response = update_status(
            Json(update_status_req),
            &rocket::State::from(&state),
            &rocket::State::from(&client),
        ).await;

        assert!(response.is_ok());

        // ステータスが更新されたことを確認
        let filter = doc! { "transaction_id": &test_transaction.transaction_id };
        let updated_doc = collection.find_one(filter.clone(), None).await.unwrap();

        // トランザクションがオペレーショナルDBから削除されていることを確認
        assert!(updated_doc.is_none());

        // 分析用データベースにトランザクションが移動していることを確認
        // `mongo_config.json` から analytics_uri を取得
        let mongo_config = load_mongo_config().expect("Failed to load mongo_config.json");
        let analytics_uri = mongo_config.analytics.get(&test_transaction.sender_continent)
            .or_else(|| mongo_config.analytics.get("Default"))
            .expect("Analytics URI not found in configuration")
            .clone();

        let analytics_client = MongoClient::with_uri_str(&analytics_uri).await.unwrap();
        let analytics_db_name = {
            let uri_parts: Vec<&str> = analytics_uri.split('/').collect();
            uri_parts.last().unwrap_or("analytics_db")
        };
        let analytics_db = analytics_client.database(analytics_db_name);
        let analytics_collection = analytics_db.collection::<Document>("transactions");

        let migrated_doc = analytics_collection.find_one(filter, None).await.unwrap();
        assert!(migrated_doc.is_some());

        // テストデータのクリーンアップ
        analytics_collection.delete_one(doc! { "transaction_id": &test_transaction.transaction_id }, None).await.unwrap();
    }
}