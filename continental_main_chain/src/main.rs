use rocket::{get, post, routes, serde::json::Json, State};
use serde_json::Value;
use rocket::serde::{Deserialize, Serialize};
use rocket::http::Status;
use rocket::config::{Config, TlsConfig};
use rocket::config::LogLevel;  // 追加
use tokio::sync::Mutex; // TokioのMutexをインポート
use tokio_stream::StreamExt;  // このインポートを追加
use tokio::time::Duration; // 追加
use tokio::time::sleep; // sleep関数をインポート
use chrono::{DateTime, Utc};
use reqwest::Client;
use sha2::{Sha256, Digest}; 
use hex;
use base64;
use std::collections::HashMap;
use std::sync::Arc;
use std::io::BufReader;
use std::error::Error;
use std::fs;
use std::fs::File; 
use mongodb::{bson::doc, Client as MongoClient, Collection, options::ClientOptions}; // MongoDB関連のクレートをインポート
use mongodb::bson::Document;
use mongodb::options::IndexOptions;
use mongodb::IndexModel;
use bson::DateTime as BsonDateTime;
use bson::Bson;
use immudb_proto::{SetRequest, KeyValue};
use immudb_proto::immudb_proto_function;
use prost::bytes::Bytes;

#[derive(Debug, Clone)]
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

// 各都市の設定
#[derive(Debug, Deserialize, Clone)]
struct City {
    name: String,
    city_port: String,
    city_flask_port: String,
}

// デフォルト設定
#[derive(Debug, Deserialize, Clone)]
struct DefaultConfig {
    mongodb_port: String,
    flask_port: String,
    cities: Vec<City>,
}

// 大陸ごとの設定
#[derive(Debug, Deserialize, Clone)]
struct ContinentConfig {
    mongodb_port: String,
    flask_port: String,
    cities: Vec<City>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    index: u64,
    timestamp: DateTime<Utc>,
    data: String,
    prev_hash: String,
    hash: String,
    verifiable_credential: String,
    signature: String,  // Vec<u8> から String に変更
}

#[derive(Debug, Deserialize)]
struct ImmudbConfig {
    host: String,
    port: u16,
}
#[derive(Debug, Deserialize)]
struct Config {
    continents: HashMap<String, ImmudbConfig>,
    default: ImmudbConfig,
}

impl Block {
    pub fn create_new_block(transactions: &[Transaction], prev_hash: &str) -> Block {
        // トランザクションデータをJSON文字列に変換
        let data = serde_json::to_string(transactions).unwrap_or_default();

        // 現在のタイムスタンプを取得
        let timestamp = Utc::now();

        // ブロックのハッシュを生成
        let hash = format!("{:x}", md5::compute(format!("{}{}{}", prev_hash, timestamp, data)));

        // サンプルの署名をVec<u8>として生成し、それをStringに変換
        let signature_vec = b"sample_signature".to_vec();
        let signature = base64::encode(signature_vec); // base64エンコードでVec<u8>からStringに変換

        Block {
            index: 0, // インデックスは適切に設定してください
            timestamp,
            data,
            prev_hash: prev_hash.to_string(),
            hash,
            verifiable_credential: "example_credential".to_string(), // 適切な値を設定
            signature, // String型の署名
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f64,
    verifiable_credential: String,
    signature: String,  // Vec<u8> から String に変更
    transaction_id: String,
    timestamp: String,
    subject: String,           // 追加
    action_level: String,      // 追加
    dimension: String,         // 追加
    fluctuation: String,       // 追加
    organism_name: String,     // 追加
    details: String,           // 追加
    goods_or_money: String,    // 追加
    transaction_type: String,  // 追加
    sender_municipality: String,  // 追加
    receiver_municipality: String, // 追加
    sender_continent: String,  // 追加
    receiver_continent: String, // 追加
    status: String,             // 新しく追加
    created_at: DateTime<Utc>,  // 新しく追加
    sender_municipal_id: String,  // 追加
    receiver_municipal_id: String, // 追加
}

#[derive(Debug, Deserialize, Serialize)]
struct CompleteTransactionRequest {
    transaction_id: String,
    new_status: String, // 状態を変更する際の新しいステータス
}

#[derive(Serialize, Deserialize)]
struct GossipRequest {
    transactions: HashMap<String, Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GossipBlockRequest {
    blocks: Vec<Block>,
}

#[derive(Serialize)]
struct UpdateStatusRequest {
    transaction_id: String,
    new_status: String,
    municipal_id: String,
    continent: String,
}

// 大陸のURLをマッピングするハッシュマップ
lazy_static! {
    static ref CONTINENTAL_CHAIN_URLS: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert("Asia".to_string(), "http://localhost:8101".to_string());
        m.insert("Europe".to_string(), "http://localhost:8102".to_string());
        // 他の大陸も同様に追加
        m
    };
}

//　データをロードする関数
fn load_municipalities_data() -> Result<HashMap<String, ContinentConfig>, Box<dyn Error>> {
    let file_path = "D:\\city_chain_project\\dapps\\municipalities.json";
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open municipalities.json: {:?}", e))?;
    let reader = BufReader::new(file);
    
    // HashMap<String, ContinentConfig>に直接デシリアライズ
    let municipalities_data: HashMap<String, ContinentConfig> = serde_json::from_reader(reader)
        .map_err(|e| format!("Failed to parse municipalities.json: {:?}", e))?;
    Ok(municipalities_data)
}

fn load_immudb_config() -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open("immudb_config.json")?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    Ok(config)
}

fn get_immudb_address(config: &Config, continent: &str) -> (String, u16) {
    if let Some(immudb_config) = config.continents.get(continent) {
        (immudb_config.host.clone(), immudb_config.port)
    } else {
        (config.default.host.clone(), config.default.port)
    }
}

fn determine_municipal_chain_url(municipality: &str) -> Option<String> {
    // 'continent-city' 形式で大陸と市町村名を分離
    let parts: Vec<&str> = municipality.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let continent = parts[0];
    let city_name = parts[1];

    // municipalities.jsonデータをロード
    let municipalities_data = load_municipalities_data().ok()?;

    // 大陸データから該当する市町村のURLを取得
    let continent_config = municipalities_data.get(continent)
    .or_else(|| municipalities_data.get("Default"))?;  // "Default"をフォールバック

    // 都市を検索
    for city in &continent_config.cities {
    if city.name == city_name {
        return Some(format!("http://localhost:{}", city.city_flask_port));
    }
}

    // 該当するデータがなければ None を返す
    None
}

fn get_continental_chain_url(continent: &str) -> Option<String> {
    let municipalities_data = load_municipalities_data().ok()?;

    // `continents` フィールドを参照せず、直接大陸名で取得
    let continent_config = municipalities_data.get(continent)
        .or_else(|| municipalities_data.get("Default"))?;

    // flask_portを使ってURLを生成
    Some(format!("http://localhost:{}", continent_config.flask_port))
}

fn get_flask_port(continent: &str) -> u16 {
    // municipalities.json データをロード
    let municipalities_data = load_municipalities_data().expect("Failed to load municipalities data");

    // 指定された大陸のデータを直接取得
    if let Some(continent_data) = municipalities_data.get(continent) {
        // flask_port を文字列から数値に変換して返す
        return continent_data.flask_port.parse::<u16>().unwrap_or(1034);
    }

    // デフォルトの大陸のデータを取得
    if let Some(default_data) = municipalities_data.get("Default") {
        return default_data.flask_port.parse::<u16>().unwrap_or(1034);
    }

    // それでも取得できない場合は、デフォルトのポートを返す
    1034
}

// Blockchain型の定義
type Blockchain = Arc<Mutex<Vec<Block>>>;
type PendingTransactions = Arc<Mutex<HashMap<String, Transaction>>>;

struct DPoS {
    user_evaluations: HashMap<String, EvaluationItems>,
    municipal_representatives: HashMap<String, Vec<Representative>>,
    approved_representative: Option<String>,
}

impl DPoS {
    fn new() -> Self {
        let mut representatives = HashMap::new();
        // 代表者リストを初期化（例としてハードコード）
        representatives.insert("Asia-Tokyo".to_string(), vec!["Rep1".to_string(), "Rep2".to_string()]);
        representatives.insert("Europe-London".to_string(), vec!["Rep3".to_string(), "Rep4".to_string()]);
        // 他の市町村も同様に追加

        println!("Initializing DPoS...");
        DPoS {
            user_evaluations: HashMap::new(),
            municipal_representatives: HashMap::new(),
            approved_representative: None,
        }
    }

    /// ユーザーIDから市町村を取得する関数
    fn get_user_municipality(&self, user_id: &str) -> String {
        // 例としてユーザーIDが "Asia-Tokyo-User1" の形式であると仮定
        let parts: Vec<&str> = user_id.split('-').collect();
        if parts.len() >= 2 {
            format!("{}-{}", parts[0], parts[1])
        } else {
            "Unknown-Municipality".to_string()
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

    /// 愛貨消費による貢献度を計算する関数
    fn calculate_contribution(&self, transaction: &Transaction) -> f64 {
        // 実際のロジックをここに実装
        // ここでは仮にトランザクションの金額をそのまま返す
        transaction.amount
    }

    /// ユーザー評価データを更新する関数
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

    /// 市町村の代表者を選出する関数
    async fn elect_representative(&mut self, municipality: &str) {
        println!("Electing representative for municipality: {}", municipality);

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

    /// 大陸代表者を選出する関数
    fn select_continental_representatives(&mut self) {
        // 市町村代表者を集める
        let mut continental_candidates: Vec<(String, f64)> = Vec::new();

        for representatives in self.municipal_representatives.values() {
            for rep in representatives {
                if rep.start_date <= Utc::now() && Utc::now() <= rep.end_date {
                    // 評価項目の合計値を計算
                    if let Some(eval) = self.user_evaluations.get(&rep.user_id) {
                        let total_score = eval.total_love_token_usage + eval.value_from_tokens_received + eval.contribution_by_token_usage;
                        continental_candidates.push((rep.user_id.clone(), total_score));
                    }
                }
            }
        }

        // スコアの高い順にソート
        continental_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).reverse());

        // 上位5名を選出（実際には大陸ごとに分ける必要があります）
        let top_5_continental_reps = continental_candidates.into_iter().take(5).collect::<Vec<_>>();

        // 大陸代表者リストを更新（フィールドを追加するか、別の方法で管理）
        // self.continental_representatives = top_5_continental_reps;

        // 新しい大陸代表者にメッセージを送信
        for (user_id, _) in top_5_continental_reps {
            self.send_message_to_user(&user_id, "大陸代表に選出されました。よろしくお願いします。");
        }
    }

    /// 代表者の選出を実行する関数
    fn perform_selection(&mut self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) {
        // 評価期間内のデータを使用（実際の実装ではフィルタリングが必要）

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
            rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).reverse());

            // 上位5名を選出
            let top_5 = rankings.into_iter().take(5);

            let representatives = top_5.map(|(user_id, _)| Representative {
                user_id,
                start_date: end_date + Duration::days(90), // 任期開始は3ヶ月後
                end_date: end_date + Duration::days(90 * 2), // 任期はその3ヶ月後まで
            }).collect::<Vec<_>>();

            self.municipal_representatives.insert(municipality.clone(), representatives);

            // 新しい代表者にメッセージを送信
            for rep in self.municipal_representatives.get(&municipality).unwrap() {
                self.send_message_to_user(&rep.user_id, &format!("{} から {} までの期間、{} の代表よろしくお願いします", rep.start_date, rep.end_date, municipality));
            }
        }
        
        // ブロックを承認する関数
        fn approve_block(&self, block: &Block) -> bool {
            // 大陸代表者のリストから承認者を選出し、承認を行います
            println!("ブロック承認: 大陸代表者による承認を行います。");
            // 承認ロジックを実装してください
            true
        }
    }

    /// ユーザーにメッセージを送信する関数
    fn send_message_to_user(&self, user_id: &str, message: &str) {
        // メッセージ送信ロジックを実装
        println!("メッセージ送信先: {} 内容: {}", user_id, message);
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

impl AppState {
    async fn create_indexes(&self) {
        // "status"フィールドのインデックス作成
        let status_index = IndexModel::builder()
            .keys(doc! { "status": 1 })  // インデックスを作成するフィールドを指定
            .options(Some(IndexOptions::builder().build()))  // オプションの設定（今回はデフォルト）
            .build();

        self.mongo_collection.create_index(status_index, None)
            .await
            .expect("Failed to create index on status");

        // "created_at"フィールドのインデックス作成
        let created_at_index = IndexModel::builder()
            .keys(doc! { "created_at": 1 })
            .options(Some(IndexOptions::builder().build()))
            .build();

        self.mongo_collection.create_index(created_at_index, None)
            .await
            .expect("Failed to create index on created_at");

        println!("Indexes created on 'status' and 'created_at' fields.");
    }
}

impl Consensus {
    fn new(fixed_representative: String) -> Self {
        Consensus {
            dpos: DPoS::new(fixed_representative),
            poh: ProofOfHistory::new(),
            transactions: Vec::new(),
        }
    }

    fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    fn process_transactions(&mut self) {
        for transaction in &mut self.transactions {
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
    fn generate_signature(&mut self, private_key: &str) {
        use ntru::crypto::NtruSign;

        let priv_key = NtruSign::from_bytes(private_key.as_bytes()).expect("Invalid private key");
        let signature = priv_key.sign(self.data.as_bytes());

        self.signature = base64::encode(signature);
    }

    fn generate_proof_of_history(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}{:?}", self.sender, self.timestamp).as_bytes());
        hex::encode(hasher.finalize())
    }

    fn verify_signature(&self, public_key: &str) -> bool {
        // 署名の検証ロジック
        !self.signature.is_empty() && public_key == "public_key" // 仮の検証
    }
}

#[derive(Clone)]
struct AppState {
    pending_transactions: Arc<Mutex<HashMap<String, Transaction>>>, // 保留トランザクションリストをArcでラップ
    other_continental_chains: Arc<Mutex<Vec<String>>>, // 他のContinental ChainのURLリストをArc<Mutex<Vec<String>>>に修正
    mongo_collection: Collection<Transaction>, // トランザクションのコレクション
    block_collection: Collection<Document>, // ブロックのコレクション
    blockchain: Arc<Mutex<Vec<Block>>>,    // ブロックチェーンを管理
    municipal_chain_urls: HashMap<String, String>, // Municipal ChainのURL
    dpos: Arc<Mutex<DPoS>>,
}

fn calculate_block_hash(index: u64, prev_hash: &str, timestamp: &DateTime<Utc>, data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(index.to_string());
    hasher.update(prev_hash);
    hasher.update(timestamp.to_rfc3339());
    hasher.update(data);
    hex::encode(hasher.finalize())
}


// 新しくルートを追加
#[get("/")]
fn index() -> &'static str {
    "Welcome to the Continental Main Chain!"
}

fn load_mongodb_config() -> Result<Value, Box<dyn std::error::Error>> {
    let file_path = "D:\\city_chain_project\\mongodb_config.json";
    let file_content = fs::read_to_string(file_path)?;
    let json_data: Value = serde_json::from_str(&file_content)?;
    Ok(json_data)
}

fn get_mongo_uri(instance_type: &str, continent: &str) -> Result<String, String> {
    let municipalities_data = load_municipalities_data().map_err(|e| format!("Failed to load municipalities data: {:?}", e))?;

    let continent_config = municipalities_data.get(continent)
        .ok_or_else(|| format!("Continent '{}' not found in municipalities data", continent))?;
    
    let uri = &continent_config.mongodb_port;  // Use the mongodb_port directly from continent config
    Ok(format!("mongodb://localhost:{}", uri))
}

// MongoDBへのトランザクション保存処理
async fn save_transaction_to_mongo(transaction: &Transaction, collection: &Collection<Document>) -> Result<(), mongodb::error::Error> {
    let doc = doc! {
        "transaction_id": &transaction.transaction_id,
        "sender": &transaction.sender,
        "receiver": &transaction.receiver,
        "amount": &transaction.amount,
        "verifiable_credential": &transaction.verifiable_credential,
        "signature": &transaction.signature,
        "timestamp": &transaction.timestamp,
        "subject": &transaction.subject,
        "action_level": &transaction.action_level,
        "dimension": &transaction.dimension,
        "fluctuation": &transaction.fluctuation,
        "organism_name": &transaction.organism_name,
        "details": &transaction.details,
        "goods_or_money": &transaction.goods_or_money,
        "transaction_type": &transaction.transaction_type,
        "sender_municipality":&transaction.sender_municipality,
        "receiver_municipality": &transaction.receiver_municipality,
        "sender_continent": &transaction.sender_continent,
        "receiver_continent": &transaction.receiver_continent,
        "status": &transaction.status,
        "created_at": bson::DateTime::from_millis(transaction.created_at.timestamp_millis()),  // 修正部分
        "sender_municipal_id": &transaction.sender_municipal_id,  // 追加
        "receiver_municipal_id": &transaction.receiver_municipal_id,  // 追加
    };

    match collection.insert_one(doc, None).await {
        Ok(_) => {
            println!("Transaction saved successfully to MongoDB.");
            Ok(())
        },
        Err(e) => {
            println!("Failed to save transaction to MongoDB: {:?}", e);
            Err(e)
        },
    }
}

async fn get_transactions_by_municipal_id(
    municipal_id: &str,
    collection: &Collection<Transaction>,
) -> Result<Vec<Transaction>, mongodb::error::Error> {
    let filter = doc! {
        "$or": [
            { "sender_municipal_id": municipal_id },
            { "receiver_municipal_id": municipal_id },
        ]
    };
    let mut cursor = collection.find(filter, None).await?;
    let mut transactions = Vec::new();

    while let Some(transaction) = cursor.try_next().await? {
        transactions.push(transaction);
    }

    Ok(transactions)
}

#[post("/receive_transaction", format = "json", data = "<transaction_json>")]
async fn receive_transaction(
    transaction_json: Json<Transaction>,
    state: &State<Arc<Mutex<AppState>>>,
    client: &State<Client>,
) -> Result<Status, String> {
    let transaction = transaction_json.into_inner();

    println!("Received transaction: {:?}", transaction.transaction_id);

    // DPoS のロックを取得
    let mut dpos_guard = state.lock().await.dpos.lock().await;

    // トランザクションに基づいてユーザー評価データを更新
    dpos_guard.update_user_evaluations(&transaction);

    // 送信者の市町村で代表者を選出
    dpos_guard.elect_representative(&transaction.sender_municipality);

    // トランザクションの承認
    if !dpos_guard.approve_transaction(&transaction) {
        println!("Transaction approval failed by DPoS.");
        return Err("Transaction not approved by DPoS.".to_string());
    }

    // 署名の検証（必要に応じて実装）
    if !transaction.verify_signature("public_key") {
        let err_msg = format!("Invalid signature for transaction: {:?}", transaction.transaction_id);
        println!("{}", err_msg);
        return Err(err_msg);
    }

    // トランザクションの検証
    if let Err(e) = validate_transaction(&transaction) {
        println!("Transaction validation failed: {}", e);
        return Err(format!("Transaction validation failed: {}", e));
    }

    // 受信者の大陸を判別
    let receiver_continent = transaction.receiver_continent.clone();
    let current_continent = CURRENT_CONTINENT.to_string(); // CURRENT_CONTINENT は定数として定義

    // 自分の大陸と異なる場合、該当する大陸にトランザクションを転送
    if receiver_continent != current_continent {
        if let Some(url) = CONTINENTAL_CHAIN_URLS.get(&receiver_continent) {
            println!("Forwarding transaction to {} continent", receiver_continent);

            // 他の大陸の continental_main_chain にトランザクションを転送
            let res = client.post(&format!("{}/receive_transaction", url))
                .json(&transaction)
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Transaction forwarded successfully.");
                        Ok(Status::Accepted)
                    } else {
                        let err_msg: String = response.text().await.unwrap_or("Unknown error".into());
                        println!("Failed to forward transaction: {}", err_msg);
                        Err(format!("Failed to forward transaction: {}", err_msg))
                    }
                },
                Err(err) => {
                    println!("Failed to forward transaction: {:?}", err);
                    Err(format!("Failed to forward transaction: {:?}", err))
                },
            }
        } else {
            println!("Unknown receiver continent: {}", receiver_continent);
            Err("Unknown receiver continent.".to_string())
        }
    } else {
        // 自分の大陸内のトランザクションの場合、既存の処理を行う
        let state_guard = state.lock().await;

        // timestamp と created_at を BSON の DateTime に変換
        let bson_timestamp = match DateTime::parse_from_rfc3339(&transaction.timestamp) {
            Ok(dt) => BsonDateTime::from_millis(dt.timestamp_millis()),
            Err(e) => {
                println!("Failed to parse timestamp: {}", e);
                return Err("Invalid timestamp format".to_string());
            }
        };

        let bson_created_at = BsonDateTime::from_millis(transaction.created_at.timestamp_millis());

        // MongoDBにトランザクションを保存
        let doc = doc! {
            "transaction_id": &transaction.transaction_id,
            "sender": &transaction.sender,
            "receiver": &transaction.receiver,
            "amount": &transaction.amount,
            "verifiable_credential": &transaction.verifiable_credential,
            "signature": &transaction.signature,
            "timestamp": bson_timestamp,
            "subject": &transaction.subject,
            "action_level": &transaction.action_level,
            "dimension": &transaction.dimension,
            "fluctuation": &transaction.fluctuation,
            "organism_name": &transaction.organism_name,
            "details": &transaction.details,
            "goods_or_money": &transaction.goods_or_money,
            "transaction_type": &transaction.transaction_type,
            "sender_municipality": &transaction.sender_municipality,
            "receiver_municipality": &transaction.receiver_municipality,
            "sender_continent": &transaction.sender_continent,
            "receiver_continent": &transaction.receiver_continent,
            "status": &transaction.status,
            "created_at": bson_created_at,
            "sender_municipal_id": &transaction.sender_municipal_id,
            "receiver_municipal_id": &transaction.receiver_municipal_id,
        };

        match state_guard.mongo_collection.insert_one(doc, None).await {
            Ok(_) => println!("Transaction inserted successfully."),
            Err(err) => {
                println!("Failed to insert transaction: {:?}", err);
                return Err(format!("MongoDB Insert Error: {:?}", err));
            }
        }

        // トランザクションを保留リストに追加
        {
            let mut pending_transactions_guard = state_guard.pending_transactions.lock().await;
            pending_transactions_guard.insert(transaction.transaction_id.clone(), transaction.clone());
            println!("Transaction added to pending list: {:?}", transaction.transaction_id);
        }

        // トランザクションタイプに応じて処理
        match transaction.transaction_type.as_str() {
            "send" => {
                // 送信トランザクションの処理
                // ステータスを "send_complete" に更新
                let update_status_req = UpdateStatusRequest {
                    transaction_id: transaction.transaction_id.clone(),
                    new_status: "send_complete".to_string(),
                    municipal_id: transaction.sender_municipal_id.clone(),
                    continent: transaction.sender_continent.clone(),
                };

                // `/update_status` エンドポイントを呼び出してステータスを更新
                let res = client.post(&format!("{}/update_status", MUNICIPAL_CHAIN_URL))
                    .json(&update_status_req)
                    .send()
                    .await;

                match res {
                    Ok(response) => {
                        if response.status().is_success() {
                            println!("Transaction approved and status updated to 'send_complete'.");
                            Ok(Status::Accepted)
                        } else {
                            let err_msg: String = response.text().await.unwrap_or("Unknown error".into());
                            println!("Failed to update transaction status: {}", err_msg);
                            Err(format!("Failed to update transaction status: {}", err_msg))
                        }
                    },
                    Err(err) => {
                        println!("Failed to call /update_status endpoint: {:?}", err);
                        Err(format!("Failed to call /update_status endpoint: {:?}", err))
                    },
                }
            },

            "receive" => {
                // 受信トランザクションの処理
                // ステータスを "complete" に更新
                let update_status_req = UpdateStatusRequest {
                    transaction_id: transaction.transaction_id.clone(),
                    new_status: "complete".to_string(),
                    municipal_id: transaction.receiver_municipal_id.clone(),
                    continent: transaction.receiver_continent.clone(),
                };

                // `/update_status` エンドポイントを呼び出してステータスを更新
                let res = client.post(&format!("{}/update_status", MUNICIPAL_CHAIN_URL))
                    .json(&update_status_req)
                    .send()
                    .await;

                match res {
                    Ok(response) => {
                        if response.status().is_success() {
                            println!("Transaction approved and status updated to 'complete'.");
                            Ok(Status::Accepted)
                        } else {
                            let err_msg: String = response.text().await.unwrap_or("Unknown error".into());
                            println!("Failed to update transaction status: {}", err_msg);
                            Err(format!("Failed to update transaction status. Error: {}", err_msg))
                        }
                    },
                    Err(err) => {
                        println!("Failed to call /update_status endpoint: {:?}", err);
                        Err(format!("HTTP Request Error: {:?}", err))
                    },
                }
            },
            _ => {
                println!("Invalid transaction type: {}", transaction.transaction_type);
                Err("Invalid transaction type.".to_string())
            }
        }
    }
}

async fn update_transaction_status(
    transaction_id: &str,
    municipal_id: &str,
    new_status: &str,
    collection: &Collection<Transaction>,
) -> Result<UpdateResult, mongodb::error::Error> {
    let filter = doc! {
        "transaction_id": transaction_id,
        "$or": [
            { "sender_municipal_id": municipal_id },
            { "receiver_municipal_id": municipal_id },
        ]
    };
    let update = doc! { "$set": { "status": new_status } };
    let result = collection.update_one(filter, update, None).await?;

    if result.matched_count == 0 {
        log::warn!("Transaction not found: {}", transaction_id);
    } else {
        log::info!("Transaction status updated: {}", transaction_id);
    }

    Ok(result)
}

#[post("/update_status", format = "json", data = "<status_update>")]
async fn update_status(
    status_update: Json<StatusUpdateRequest>,
    state: &State<AppState>,
) -> Result<status::Accepted<String>, status::BadRequest<String>> {
    let update_request = status_update.into_inner();

    let result = update_transaction_status(
        &update_request.transaction_id,
        &update_request.municipal_id,
        &update_request.new_status,
        &state.mongo_collection,
    ).await;

    match result {
        Ok(update_result) => {
            if update_result.matched_count > 0 {
                println!("Transaction status updated: {}", update_request.transaction_id);
                Ok(status::Accepted(Some("Status updated".to_string())))
            } else {
                Err(status::BadRequest(Some("Transaction not found".to_string())))
            }
        },
        Err(e) => Err(status::BadRequest(Some(format!("Database error: {}", e)))),
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

fn validate_transaction(transaction: &Transaction) -> Result<(), String> {
    // 必要なフィールドが存在するかを確認
    if transaction.sender_municipal_id.is_empty() {
        return Err("sender_municipal_id is missing".to_string());
    }
    if transaction.receiver_municipal_id.is_empty() {
        return Err("receiver_municipal_id is missing".to_string());
    }
    if transaction.sender.is_empty() {
        return Err("sender is missing".to_string());
    }
    if transaction.receiver.is_empty() {
        return Err("receiver is missing".to_string());
    }
    if transaction.amount <= 0.0 {
        return Err("amount must be greater than zero".to_string());
    }
    // 他の検証ロジック
    Ok(())
}

async fn process_pending_transactions(state: Arc<Mutex<AppState>>) {
    loop {
        // 一定時間待機（例えば10秒）
        sleep(Duration::from_secs(10)).await;

        // 保留トランザクションを取得
        let transactions_to_process = {
            let state_guard = state.lock().await;
            let mut pending_transactions_guard = state_guard.pending_transactions.lock().await;
            
            // トランザクションが5件以上ある場合、すべて取得
            if pending_transactions_guard.len() >= 5 {
                let txs: Vec<Transaction> = pending_transactions_guard.values().cloned().collect();
                pending_transactions_guard.clear();
                txs
            } else {
                continue; // 5件未満の場合は次のループへ
            }
        };

        // ブロックチェーンにアクセス
        let mut state_guard = state.lock().await; // 再度ロックを取得
        let mut blockchain_guard = state_guard.blockchain.lock().await;

        // 新しいブロックのインデックスと前のブロックのハッシュを設定
        let index = blockchain_guard.len() as u64;
        let prev_hash = if let Some(last_block) = blockchain_guard.last() {
            last_block.hash.clone()
        } else {
            "genesis".to_string()
        };

        // ブロックを生成
        let new_block = Block {
            index,
            timestamp: Utc::now(),
            data: serde_json::to_string(&transactions_to_process).unwrap(),
            prev_hash,
            hash: "temporary_hash".to_string(), // 実際のハッシュ計算をここに実装
            verifiable_credential: "credential".to_string(),
            signature: base64::encode(vec![]), // 必要に応じて実際の署名を追加
        };

        // 生成したブロックをブロックチェーンに追加
        blockchain_guard.push(new_block.clone());

        // MongoDB にブロックを保存
        {
            let state_guard = state.lock().await;
            let block_doc = doc! {
                "index": new_block.index as i64, // u64 を i64 に変換
                "timestamp": BsonDateTime::from_millis(new_block.timestamp.timestamp_millis()),
                "data": new_block.data.clone(),
                "prev_hash": new_block.prev_hash.clone(),
                "hash": new_block.hash.clone(),
                "verifiable_credential": new_block.verifiable_credential.clone(),
                "signature": base64::encode(new_block.signature.clone()), // Vec<u8>をBase64エンコードしてStringに変換
            };

            match state_guard.block_collection.insert_one(block_doc, None).await {
                Ok(_) => println!("Block saved to MongoDB."),
                Err(err) => println!("Failed to save block to MongoDB: {:?}", err),
            }
        }

        // トランザクションのステータスを更新
        {
            let state_guard = state.lock().await;
            for tx in transactions_to_process {
                let filter = doc! { "transaction_id": &tx.transaction_id };
                let update = doc! { "$set": { "status": "completed" } };
                if let Err(e) = state_guard.mongo_collection.update_one(filter, update, None).await {
                    println!("Failed to update transaction status for {}: {:?}", tx.transaction_id, e);
                }
            }
        }
    }
}

#[post("/transaction", format = "json", data = "<transaction>")]
async fn create_transaction(
    transaction: Json<Transaction>,
    state: &State<Arc<Mutex<AppState>>>, 
    client: &State<Client>
) -> Result<Status, String> {
    let mut transaction = transaction.into_inner();

    // トランザクションのバリデーションを追加
    if let Err(e) = validate_transaction(&transaction) {
        log::error!("Transaction validation failed: {}", e);
        return Err(format!("Transaction validation failed: {}", e));
    }

    // `status`と`created_at`を設定
    transaction.status = match transaction.transaction_type.as_str() {
        "send" => "send_pending".to_string(),
        "receive" => "receive_pending".to_string(),
        _ => "unknown".to_string(),
    };
    transaction.created_at = Utc::now();

    println!("Received transaction: {:?}", transaction);

    // 署名の検証
    if !transaction.verify_signature("public_key") {
        let err_msg = format!("Invalid signature for transaction: {:?}", transaction.transaction_id);
        println!("{}", err_msg);
        return Err(err_msg);
    }

    let state_guard = state.lock().await;

    // chrono::DateTime<Utc> を BSON DateTime に変換
    let bson_timestamp = BsonDateTime::from_millis(transaction.timestamp.parse::<DateTime<Utc>>().unwrap().timestamp_millis());
    let bson_created_at = BsonDateTime::from_millis(transaction.created_at.timestamp_millis());

    // MongoDBにトランザクションを保存
    let doc = doc! {
        "transaction_id": &transaction.transaction_id,
        "sender": &transaction.sender,
        "receiver": &transaction.receiver,
        "amount": &transaction.amount,
        "verifiable_credential": &transaction.verifiable_credential,
        "signature": &transaction.signature,
        "timestamp": bson_timestamp,
        "subject": &transaction.subject,
        "action_level": &transaction.action_level,
        "dimension": &transaction.dimension,
        "fluctuation": &transaction.fluctuation,
        "organism_name": &transaction.organism_name,
        "details": &transaction.details,
        "goods_or_money": &transaction.goods_or_money,
        "transaction_type": &transaction.transaction_type,
        "sender_municipality": &transaction.sender_municipality,
        "receiver_municipality": &transaction.receiver_municipality,
        "sender_continent": &transaction.sender_continent,
        "receiver_continent": &transaction.receiver_continent,
        "status": &transaction.status,
        "created_at": bson_created_at,
        "sender_municipal_id": &transaction.sender_municipal_id,  // 追加
        "receiver_municipal_id": &transaction.receiver_municipal_id,  // 追加
    };

    match state_guard.mongo_collection.insert_one(doc, None).await {
        Ok(_) => println!("Transaction inserted successfully with status 'send_pending'."),
        Err(err) => {
            println!("Failed to insert transaction: {:?}", err);
            return Err("Internal Server Error".to_string());
        }
    }

    // トランザクションを保留リストに追加
    {
        let mut pending_transactions_guard = state_guard.pending_transactions.lock().await;
        pending_transactions_guard.insert(transaction.transaction_id.clone(), transaction.clone());
        println!("Transaction added to pending list: {:?}", transaction.transaction_id);
    }

    Ok(Status::Accepted)
}

#[post("/update_status", format = "json", data = "<update_request>")]
async fn update_status(
    update_request: Json<UpdateStatusRequest>,
    state: &State<Arc<Mutex<AppState>>>,
) -> Result<Status, Status> {
    let update_request = update_request.into_inner();
    let transaction_id = update_request.transaction_id;
    let new_status = update_request.new_status;

    let state_guard = state.lock().await;
    let filter = doc! { "transaction_id": &transaction_id };
    let transaction = state_guard.mongo_collection.find_one(filter.clone(), None).await?;

    if transaction.is_none() {
        return Err(Status::NotFound);
    }

    let current_status = transaction.get_str("status").unwrap_or("");

    if current_status != "pending" && new_status == "complete" {
        return Err(Status::BadRequest(Some("Invalid status transition".to_string())));
    }

    let update = doc! { 
        "$set": { 
            "status": &new_status, 
            "updated_at": BsonDateTime::from_millis(Utc::now().timestamp_millis()) 
        } 
    };

    match state_guard.mongo_collection.update_one(filter.clone(), update, None).await {
        Ok(result) => {
            if result.matched_count == 0 {
                return Err(Status::NotFound);
            }
            println!("Transaction {} updated to status {}", transaction_id, new_status);

            // ステータスが "shared" に更新された場合、分析用データベースに移行
            if new_status == "shared" {
                match state_guard.mongo_collection.find_one(filter.clone(), None).await {
                    Ok(Some(transaction_doc)) => {
                        // 分析用データベースに移行
                        let analytics_uri = "mongodb://localhost:3000";  // 分析用MongoDBのURIに変更
                        let analytics_client = MongoClient::with_uri_str(analytics_uri).await.map_err(|e| {
                            println!("Failed to connect to analytics MongoDB: {:?}", e);
                            Status::InternalServerError
                        })?;
                        let analytics_collection = analytics_client.database("analytics_db").collection::<Document>("shared_transactions");

                        match analytics_collection.insert_one(transaction_doc.clone(), None).await {
                            Ok(_) => {
                                println!("Transaction {} migrated to analytics DB.", transaction_id);
                                // オペレーショナルデータベースから削除
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

// 受信者からの問い合わせに対応してトランザクションをMunicipal Chainに戻す
#[post("/query_transaction", format = "json", data = "<query>")]
async fn handle_incoming_query(query: Json<Transaction>, state: &State<Arc<Mutex<AppState>>>, client: &State<Client>) -> Status {
    // `AppState` のロックを取得
    let state = state.lock().await;

    // トランザクションのロックを取得して処理
    let mut pending_transactions = state.pending_transactions.lock().await;
    if let Some(transaction) = pending_transactions.remove(&query.transaction_id) {
        // Municipal Chainにトランザクションを戻す処理
        let municipal_chain_url = format!("http://{}/receive_transaction", transaction.receiver);
        let res = client.post(&municipal_chain_url).json(&transaction).send().await;

        if res.is_ok() {
            Status::Accepted
        } else {
            Status::InternalServerError
        }
    } else {
        Status::NotFound
    }
}

#[post("/complete_transaction", format = "json", data = "<data>")]
async fn complete_transaction(
    data: Json<CompleteTransactionRequest>,
    state: &State<Arc<Mutex<AppState>>>,
    client: &State<Client>,
) -> Result<Status, String> {
    let transaction_id = &data.transaction_id;

    println!("Completing transaction: {:?}", transaction_id);

    // ロックを取得
    let mut state_guard = state.lock().await;

    // トランザクションを検索
    let filter = doc! { "transaction_id": transaction_id };
    let transaction_doc = match state_guard.mongo_collection.find_one(filter.clone(), None).await {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            println!("Transaction ID {} not found.", transaction_id);
            return Err("Transaction not found.".to_string());
        },
        Err(e) => {
            println!("Error fetching transaction: {:?}", e);
            return Err("Internal Server Error".to_string());
        },
    };

    // ステータスが適切かどうかを確認
    let transaction_status = transaction_doc.get_str("status").unwrap_or("");
    if transaction_status != "send_complete" {
        return Err("Transaction is not in a complete state.".to_string());
    }

    // トランザクションのデシリアライズ
    let transaction: Transaction = match bson::from_bson(Bson::Document(transaction_doc.clone())) {
        Ok(tx) => tx,
        Err(e) => {
            println!("Failed to deserialize transaction: {:?}", e);
            return Err("Internal Server Error".to_string());
        },
    };

    // ステータスを "complete" に更新
    let update = doc! { "$set": { "status": "complete" } };
    match state_guard.mongo_collection.update_one(filter.clone(), update, None).await {
        Ok(result) => {
            if result.matched_count == 0 {
                println!("Transaction ID {} not found during update.", transaction_id);
                return Err("Transaction not found.".to_string());
            }
            println!("Transaction {} status updated to 'complete'.", transaction_id);
        },
        Err(e) => {
            println!("Failed to update transaction status: {:?}", e);
            return Err("Internal Server Error".to_string());
        }
    }

    // ここで `municipal_chain` のURLを決定するロジックを実装します。
    fn load_municipalities_data() -> HashMap<String, Value> {
        // municipalities.jsonを読み込む
        let data = std::fs::read_to_string("municipalities.json").expect("Failed to read municipalities.json");
        let municipalities_data: HashMap<String, Value> = serde_json::from_str(&data).expect("Failed to parse municipalities.json");
        municipalities_data
    }

    // 市町村名に基づいて municipal_chain の URL を動的に取得する関数
    fn determine_municipal_chain_url(municipality: &str) -> String {
        // 'continent-city' 形式で大陸と市町村名を分離
        let parts: Vec<&str> = municipality.split('-').collect();
        if parts.len() != 2 {
            return "http://localhost:8000".to_string();  // フォーマットが不正の場合はデフォルトのURLを返す
        }

        let continent = parts[0];  // 大陸名を取得
        let city_name = parts[1];  // 市町村名を取得

        // municipalities.jsonデータをロード
        let municipalities_data = load_municipalities_data();

        // 大陸データから該当する市町村のURLを取得
        if let Some(continent_data) = municipalities_data.get(continent) {
            if let Some(municipalities) = continent_data.get("municipalities") {
                if let Some(municipality_data) = municipalities.get(city_name) {
                    if let Some(url) = municipality_data.get("municipal_chain_url") {
                        return url.as_str().unwrap_or("http://localhost:19999").to_string();
                    }
                }
            }
        }

        // 該当するデータがなければデフォルトのURLを返す
        "http://localhost:19999".to_string()
    }
    
    // `municipal_chain` のURLを決定
    let municipal_chain_url = determine_municipal_chain_url(&transaction.receiver_municipality);
    println!("Municipal chain URL: {}", municipal_chain_url);

    // `municipal_chain` にトランザクションを送信
    let res = client.post(&format!("{}/receive_transaction", municipal_chain_url))
        .json(&transaction)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                println!("Transaction {} successfully sent to municipal_chain at {}.", transaction_id, municipal_chain_url);
                Ok(Status::Ok)
            } else {
                let err_text = response.text().await.unwrap_or("Unknown error".to_string());
                println!("Failed to send transaction {} to municipal_chain: {}", transaction_id, err_text);
                Err(format!("Failed to send transaction: {}", err_text))
            }
        },
        Err(e) => {
            println!("Error sending transaction to municipal_chain: {:?}", e);
            Err("Failed to send transaction to municipal_chain.".to_string())
        },
    }
}

async fn approve_block(transaction_id: &str, client: &Client) -> Result<(), Box<dyn Error>> {
    // ブロック承認ロジック（実装は省略）
    // 承認が成功したらステータスを "send_complete" に更新
    let update_status_req = UpdateStatusRequest {
        transaction_id: transaction_id.to_string(),
        new_status: "send_complete".to_string(),
    };

    // 設定ファイルからURLを取得
    let config = load_mongodb_config().expect("Failed to load config");
    let update_status_url = config.get("update_status_url").and_then(|v| v.as_str()).unwrap_or("http://localhost:8101/update_status");

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

// トランザクションの完了を通知してGlobal Main Chainにデータを集約
#[post("/gossip_transactions", format = "json", data = "<gossip_data>")]
async fn gossip_transactions_handler(
    gossip_data: Json<GossipRequest>,
    state: &State<Arc<Mutex<AppState>>>,
) -> Status {
    let state = state.lock().await;  // AppStateのロックを取得
    let mut pending_transactions = state.pending_transactions.lock().await;

    // 他のチェーンからのトランザクションデータを受け取って反映
    for (tx_id, transaction) in &gossip_data.transactions {
        // トランザクションが既に存在しない場合にのみ追加
        if !pending_transactions.contains_key(tx_id) {
            pending_transactions.insert(tx_id.clone(), transaction.clone());
            println!("Gossip transaction added: {:?}", transaction);

            // MongoDBに保存する処理を追加
            if let Err(e) = save_transaction_to_mongo(&transaction, &state.mongo_collection).await {
                println!("Failed to save gossip transaction to MongoDB: {:?}", e);
                return Status::InternalServerError;
            }
        } else {
            println!("Transaction already exists, skipping: {:?}", tx_id);
        }
    }
    Status::Ok
}

// Gossipプロトコルを使って他の大陸チェーンとトランザクションを同期
async fn gossip_transactions_with_chains(state: Arc<Mutex<AppState>>, chain_urls: Vec<String>) {
    let state = state.lock().await;  // AppStateのロックを取得
    let pending_transactions = state.pending_transactions.lock().await;
    let gossip_data = GossipRequest {
        transactions: pending_transactions.clone(),
    };

    for chain_url in chain_urls {
        let client = Client::new();
        let res = client.post(&format!("{}/gossip_transactions", chain_url))
            .json(&gossip_data)
            .send()
            .await;
        
        match res {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Transaction gossip successfully processed by {}", chain_url);
                } else {
                    let err_msg: String = response.text().await.unwrap_or("Unknown error".into());
                    println!("Failed to process gossip transaction: {}", err_msg);
                }
            },
            Err(err) => {
                let err_msg = format!("Failed to reach chain: {:?}", err);
                println!("{}", err_msg);
            },
        }
    }
}

// Gossipプロトコルによるトランザクションの共有
async fn gossip_transactions(state: Arc<Mutex<AppState>>) {
    let state = state.lock().await; // AppStateのロックを取得

    let pending_transactions = state.pending_transactions.lock().await;
    let gossip_data = GossipRequest {
        transactions: pending_transactions.clone(),
    };

    // other_continental_chainsをロックして中身を取得
    let other_continental_chains = state.other_continental_chains.lock().await;

    for chain_url in other_continental_chains.iter() {
        let client = Client::new();
        let res = client.post(&format!("{}/gossip_transactions", chain_url))
            .json(&gossip_data)
            .send()
            .await;
        
        match res {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Transaction processed successfully.");
                } else {
                    let err_msg: String = response.text().await.unwrap_or("Unknown error".into());
                    println!("Failed to process transaction: {}", err_msg);
                }
            },
            Err(err) => {
                let err_msg = format!("Failed to reach continental chain: {:?}", err);
                println!("{}", err_msg);
            },
        }
    }
}

#[post("/gossip_blocks", format = "json", data = "<gossip_data>")]
async fn gossip_blocks_handler(
    gossip_data: Json<GossipBlockRequest>, 
    state: &State<Arc<Mutex<AppState>>>
) -> Status {
    let gossip_data = gossip_data.into_inner();
    
    // AppState全体のロックを取得
    let app_state = state.lock().await;
    
    // ブロックチェーンのロックを取得
    let mut blockchain = app_state.blockchain.lock().await;

    // 受け取ったブロックをブロックチェーンに追加
    for block in gossip_data.blocks {
        blockchain.push(block);
    }

    println!("Gossip blocks received and added to blockchain.");
    Status::Ok
}

#[get("/pending_transactions")]
async fn get_pending_transactions(state: &rocket::State<Arc<Mutex<HashMap<String, Transaction>>>>) -> Json<Vec<Transaction>> {
    let pending_transactions = state.lock().await;
    Json(pending_transactions.values().cloned().collect())
}

async fn generate_block(
    state: &State<Arc<Mutex<AppState>>>,
    client: &Client,
) -> Result<(), String> {
    // 保留中のトランザクションを取得
    let pending_transactions = {
        let state_guard = state.lock().await;
        let pending_transactions = state_guard.pending_transactions.lock().await;
        pending_transactions.clone()
    };

    if pending_transactions.is_empty() {
        println!("No pending transactions to include in the block.");
        return Ok(());
    }

    // 新しいブロックを作成
    let new_block = {
        let chain = {
            let state_guard = state.lock().await;
            state_guard.blockchain.lock().await.clone()
        };

        let last_block = chain.last().unwrap();
        let index = last_block.index + 1;
        let timestamp = Utc::now();

        // トランザクションデータをシリアライズ
        let data = serde_json::to_string(&pending_transactions.values().collect::<Vec<_>>())
            .map_err(|e| e.to_string())?;

        let prev_hash = last_block.hash.clone();

        // ブロックのハッシュを計算
        let hash = calculate_hash(index, &timestamp, &data, &prev_hash);

        Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash,
            verifiable_credential: String::new(),
            signature: vec![],
        }
    };

    // ブロックチェーンに新しいブロックを追加
    {
        let state_guard = state.lock().await;
        let mut blockchain = state_guard.blockchain.lock().await;
        blockchain.push(new_block.clone());
        println!("New block added to the chain: {:?}", new_block.index);
    }

    // 保留中のトランザクションをクリア
    {
        let state_guard = state.lock().await;
        let mut pending_transactions = state_guard.pending_transactions.lock().await;
        pending_transactions.clear();
    }

    // 他の大陸にブロックを伝播
    for (continent, url) in CONTINENTAL_CHAIN_URLS.iter() {
        if continent != CURRENT_CONTINENT {
            println!("Propagating block to {} continent", continent);

            let res = client.post(&format!("{}/receive_block", url))
                .json(&new_block)
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Block propagated successfully to {}.", continent);
                    } else {
                        let err_msg: String = response.text().await.unwrap_or("Unknown error".into());
                        println!("Failed to propagate block to {}: {}", continent, err_msg);
                    }
                },
                Err(err) => {
                    println!("Failed to propagate block to {}: {:?}", continent, err);
                },
            }
        }
    }

    Ok(())
}

// MongoDBクライアントとコレクションの初期化
async fn init_mongo_client(chain_type: &str, continent: &str) -> Result<MongoClient, Box<dyn std::error::Error>> {
    let municipalities_data = load_municipalities_data().expect("Failed to load municipalities data");

    // 指定した大陸の設定を取得。存在しない場合は "Default" の設定を使用
    let continent_config = municipalities_data.get(continent)
    .or_else(|| municipalities_data.get("Default"))
    .expect("No valid continent configuration found.")
    .clone();

    // MongoDB のポートを取得
    let mongodb_port = &continent_config.mongodb_port;

    // MongoDB URI の設定
    let mongo_uri = format!("mongodb://localhost:{}", mongodb_port);
    let flask_port = &continent_config.flask_port;

    // MongoDB クライアントを初期化
    let mongo_client_options = ClientOptions::parse(&mongo_uri).await?;
    let mongo_client = MongoClient::with_options(mongo_client_options)?;
    Ok(mongo_client)
}

async fn get_collection(client: &MongoClient, continent: &str, chain_type: &str) -> Collection<Document> {
    // municipalities_dataをアンラップ
    let municipalities_data = load_municipalities_data().expect("Failed to load municipalities data");

    // データベース名を動的に決定
    let mongo_db_name = if let Some(continent_data) = municipalities_data.get(continent) {
        if chain_type == "municipal" {
            format!("{}_municipal_db", continent) // 例えば "Asia_municipal_db" のような名前にする
        } else if chain_type == "continental" {
            format!("{}_continental_db", continent) // 例えば "Asia_continental_db" のような名前にする
        } else {
            "default_db".to_string()
        }
    } else {
        "default_db".to_string() // デフォルトのデータベース名
    };

    // 指定されたデータベースのコレクションを取得
    let db = client.database(&mongo_db_name);
    db.collection::<Document>("pending_transactions")
}

fn get_rocket_port(continent_config: &ContinentConfig) -> u16 {
    continent_config
        .flask_port
        .parse::<u16>()
        .expect("Failed to parse port as u16")
}

async fn update_transaction_status(transaction_id: &str, new_status: &str) -> Result<(), Box<dyn std::error::Error>> {
    // MongoDB に接続
    let client_uri = "mongodb://localhost:27017/";
    let client_options = ClientOptions::parse(client_uri).await?;
    let client = Client::with_options(client_options)?;

    let original_db = client.database("original_database");
    let transactions_collection = original_db.collection::<Transaction>("transactions");

    // トランザクションを検索
    let filter = doc! { "transaction_id": transaction_id };
    if let Some(mut transaction) = transactions_collection.find_one(filter.clone(), None).await? {
        // ステータスを更新
        transaction.status = new_status.to_string();
        transactions_collection.update_one(filter.clone(), doc! { "$set": { "status": new_status } }, None).await?;

        if new_status == "complete" {
            // `analytics` に移動
            move_transaction_to_analytics(&transaction).await?;
        }
    } else {
        println!("Transaction {} not found.", transaction_id);
    }

    Ok(())
}

// Gossipプロトコルを使って他の大陸チェーンとブロックを同期
async fn gossip_blocks_with_chains(state: Arc<Mutex<AppState>>, chain_urls: Vec<String>, client: reqwest::Client) {
    let state = state.lock().await;  // AppStateのロックを取得
    let blockchain = state.blockchain.lock().await;

    // 最新のブロックをGossipデータとして構築
    let latest_block = blockchain.last().cloned();

    if let Some(block) = latest_block {
        for chain_url in chain_urls {
            let client = Client::new();
            let url = format!("{}/gossip_blocks", chain_url); // ここで一時的な値を保持
            let res = client.post(&url)
                .json(&block)
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Block gossip successfully processed by {}", chain_url);
                    } else {
                        let err_msg: String = response.text().await.unwrap_or("Unknown error".into());
                        println!("Failed to process gossip block: {}", err_msg);
                    }
                },
                Err(err) => {
                    let err_msg = format!("Failed to reach chain: {:?}", err);
                    println!("{}", err_msg);
                },
            }
        }
    } else {
        println!("No block to gossip.");
    }
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
    // ブロックを JSON にシリアライズ
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

use immudb_proto::GetRequest;

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

// インデックス作成関数の例
async fn create_indexes(collection: &Collection<mongodb::bson::Document>) {
    let index_model = mongodb::IndexModel::builder()
        .keys(doc! { "transaction_id": 1 })
        .options(mongodb::options::IndexOptions::builder().unique(true).build())
        .build();

    collection.create_index(index_model, None).await.expect("Failed to create index on transaction_id");
}

async fn forward_transaction(
    transaction: &Transaction,
    client: &Client,
    target_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.post(target_url)
        .json(&transaction)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Transaction forwarded successfully: {}", transaction.transaction_id);
        Ok(())
    } else {
        let error_text = response.text().await?;
        println!("Failed to forward transaction: {}", error_text);
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to forward transaction",
        )))
    }
}

// 分析用MongoDBクライアントとコレクションの初期化
async fn init_analytics_mongo_client() -> Collection<Document> {
    // 環境変数からMongoDBのURIを取得、設定されていない場合はデフォルトURIを使用
    let analytics_uri = std::env::var("ANALYTICS_MONGO_URI").unwrap_or("mongodb://localhost:10034".to_string());

    if !analytics_uri.starts_with("mongodb://") {
        panic!("Invalid MongoDB URI scheme. Please provide a URI starting with 'mongodb://'");
    }

    let analytics_client = MongoClient::with_uri_str(analytics_uri)
        .await
        .expect("Failed to connect to analytics MongoDB");

    let analytics_collection = analytics_client.database("analytics_db").collection::<Document>("shared_transactions");

    analytics_collection
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

// 分析用データベースのインデックス作成関数を追加
async fn create_analytics_indexes(analytics_collection: &Collection<Document>) {
    // "status"フィールドのインデックスを作成
    let status_index = IndexModel::builder().keys(doc! { "status": 1 }).build();
    analytics_collection.create_index(status_index, None)
        .await
        .expect("Failed to create index on status in analytics DB");

    // "created_at"フィールドのインデックスを作成
    let created_at_index = IndexModel::builder().keys(doc! { "created_at": 1 }).build();
    analytics_collection.create_index(created_at_index, None)
        .await
        .expect("Failed to create index on created_at in analytics DB");

    println!("Indexes created on 'status' and 'created_at' fields in analytics database.");
}

#[tokio::main]
async fn main() {
    immudb_proto_function();
    let client = reqwest::Client::new(); // クライアントの初期化

    // DPoS の初期化
    let dpos = Arc::new(Mutex::new(DPoS::new()));

    // コマンドライン引数の取得
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide a continent name as an argument.");
        std::process::exit(1);
    }
    let continent = &args[1];

    // データをロード
    let municipalities_data = match load_municipalities_data() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to load municipalities data: {}", e);
            std::process::exit(1);
        }
    };

    // 指定した大陸の設定を取得。存在しない場合は "Default" の設定を使用
    let continent_config = municipalities_data
        .get(continent) 
        .or_else(|| municipalities_data.get("Default"))
        .expect("No valid continent configuration found.")
        .clone();

    // `continent_config` を使って、MongoDBとFlaskのポートを取得
    let mongodb_port = &continent_config.mongodb_port;
    let flask_port = &continent_config.flask_port;
    let mongo_uri = format!("mongodb://localhost:{}", mongodb_port);
    
    // MongoDB クライアントの初期化
    let mongo_client = match MongoClient::with_uri_str(&mongo_uri).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to initialize MongoDB client: {}", e);
            std::process::exit(1);
        }
    };

    // データベース名の設定（必要に応じて）
    let mongo_db_name = format!("{}_db", continent); 

    // Flask ポートのパース
    let flask_port: u16 = match flask_port.parse::<u16>() {
        Ok(port) => port,
        Err(e) => {
            eprintln!("Failed to parse Flask port as u16: {}", e);
            std::process::exit(1);
        }
    };

    // 以下の部分を必要に応じて調整
    println!("MongoDB running on port: {}", mongodb_port);
    println!("Flask running on port: {}", flask_port);

    // pending_transactions の初期化
    let pending_transactions = Arc::new(Mutex::new(HashMap::<String, Transaction>::new()));

    // MongoDBコレクションを選択（保留トランザクション用）
    let mongo_collection = mongo_client
        .database(&mongo_db_name)
        .collection::<Document>("pending_transactions");

    // 他の大陸のチェーンURLを設定
    let other_continental_chains = Arc::new(Mutex::new(
        municipalities_data.iter()
            .filter(|&(name, _)| name != continent) 
            .map(|(_, config)| format!("http://localhost:{}", config.flask_port))
            .collect::<Vec<String>>()
    ));        

    // municipal_chain_urls の設定
    let municipal_chain_urls: HashMap<String, String> = continent_config.cities.iter()
        .map(|city| {
            (city.name.clone(), format!("http://localhost:{}", city.city_flask_port))
        })
        .collect();

    // ブロックチェーンの初期化
    let blockchain = Arc::new(Mutex::new(Vec::<Block>::new()));

    // ブロックのコレクションを取得
    let block_collection = mongo_client
        .database(&mongo_db_name)
        .collection::<Document>("blocks");

    // AppStateの初期化
    let state = Arc::new(Mutex::new(AppState {
        pending_transactions: Arc::new(Mutex::new(HashMap::new())),
        other_continental_chains: Arc::clone(&other_continental_chains),
        mongo_collection,
        block_collection: mongo_client.database(&mongo_db_name).collection::<Document>("blocks"),
        blockchain: Arc::new(Mutex::new(Vec::new())),
        municipal_chain_urls,
        dpos: Arc::clone(&dpos),
    }));

    // インデックスを作成
    {
        let state_guard = state.lock().await;
        state_guard.create_indexes().await;
    }

    // 期限付き削除タスクを起動
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        clean_expired_send_pending_transactions(state_clone).await;
    });

    // バッチ処理タスクを開始
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        process_pending_transactions(state_clone).await;
    });

    // 分析用MongoDBのクライアントとコレクションを初期化
    let analytics_collection = init_analytics_mongo_client().await;

    // 分析用データベースのインデックスを作成
    create_analytics_indexes(&analytics_collection).await;

tokio::spawn({
    let state_clone = Arc::clone(&state_clone);
    let client_clone = client_clone.clone();
    let other_chains_for_block_creation = Arc::clone(&other_chains_for_block_creation);

    async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;

            let should_create_block = {
                let state_guard = state_clone.lock().await;
                // MongoDBから保留中のトランザクション数を取得
                match state_guard.mongo_collection.count_documents(doc! { "status": "pending" }, None).await {
                    Ok(count) => count >= 500,
                    Err(_) => false,
                }
            };

            if should_create_block {
                println!("Creating a new block...");

                // 保留トランザクションを取得
                let pending_transactions: Vec<Transaction> = {
                    let state_guard = state_clone.lock().await;
                    let mut transactions = Vec::new();
                    let mut cursor = match state_guard.mongo_collection.find(doc! { "status": "pending" }, None).await {
                        Ok(cursor) => cursor,
                        Err(e) => {
                            println!("Failed to fetch pending transactions: {:?}", e);
                            continue;
                        },
                    };

                    while let Some(result) = cursor.next().await {
                        match result {
                            Ok(doc) => {
                                // トランザクションフィールドがトップレベルに存在すると仮定
                                if let Ok(tx) = bson::from_document::<Transaction>(doc.clone()) {
                                    transactions.push(tx);
                                } else {
                                    println!("Failed to deserialize transaction document: {:?}", doc);
                                }
                            },
                            Err(e) => {
                                println!("Error reading transaction document: {:?}", e);
                            },
                        }
                    }
                    transactions
                };

                if !pending_transactions.is_empty() {
                    // state_cloneのロックを取得
                    let state_guard = state_clone.lock().await;
                    
                    // 最新ブロックを取得して新しいブロックのprev_hashを設定
                    let prev_hash = {
                        // blockchainフィールドのロックを取得
                        let blockchain_guard = state_guard.blockchain.lock().await;
                        
                        if let Some(last_block) = blockchain_guard.last() {
                            last_block.hash.clone()
                        } else {
                            "0".to_string() // 最初のブロックの場合
                        }
                    };

                    // ブロックを生成
                    let new_block = Block::create_new_block(&pending_transactions, &prev_hash);

                    // ブロックチェーンに追加
                    {
                        // state_cloneのロックを取得
                        let state_guard = state_clone.lock().await;
                        // blockchainフィールドのロックを取得
                        let mut blockchain_guard = state_guard.blockchain.lock().await;
                        // 新しいブロックをブロックチェーンに追加
                        blockchain_guard.push(new_block.clone());
                        println!("New block added to blockchain: {:?}", new_block.index);
                    }

                    // MongoDBの保留トランザクションを完了に更新
                    {
                        let mut state_guard = state_clone.lock().await;
                        for tx in &pending_transactions {
                            let filter = doc! { "transaction_id": &tx.transaction_id, "status": "pending" };
                            let update = doc! { "$set": { "status": "complete" } }; // 一貫性のため "complete" に統一
                            if let Err(e) = state_guard.mongo_collection.update_one(filter, update, None).await {
                                println!("Failed to update transaction status: {:?}", e);
                            }
                        }
                    }

                    // Gossipを通じてブロックを共有
                    let block_clone = new_block.clone();
                    let gossip_urls = {
                        // `other_chains_for_block_creation`のロックを取得
                        let chains_guard = other_chains_for_block_creation.lock().await;
                        chains_guard.clone() // `Vec<String>` をクローン
                    };

                    for chain_url in gossip_urls.iter() {
                        // 一時的な値を変数に格納
                        let formatted_url = format!("{}/gossip_blocks", chain_url); // この値を変数に保持

                        let block_owned = block_clone.clone();
                        let client_clone_block = client_clone.clone();

                        // RequestBuilderを変数にバインドしてライフタイムを延長
                        let request_builder = client_clone_block.post(formatted_url);
                        let request_builder = request_builder.json(&block_owned);
                        let res = request_builder.send().await;

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
                }
            }
        }
    }
});

    // Rocketの設定
    let rocket_config = Config::figment()
        .merge(("port", flask_port))
        .merge(("log_level", LogLevel::Normal)) // ログレベルを追加
        .merge(("workers", 10)); // ワーカースレッド数を10に設定

    // Rocketアプリケーションの起動
    rocket::custom(rocket_config)
        .manage(client) // reqwest::ClientをRocketに渡す
        .manage(state.clone())
        .manage(dpos.clone()) // DPoS を管理状態に追加
        .mount("/", routes![
            index,
            create_transaction,
            gossip_transactions_handler,
            gossip_blocks_handler,
            complete_transaction,
            update_status,
            receive_transaction,
            handle_incoming_query,
            // 他のルートも必要に応じて追加
        ])
        .launch()
        .await
        .expect("Failed to launch Rocket");
}

fn load_test_config() -> Config {
    // テスト用の設定値を定義します
    let figment = Config::figment()
        .merge(("port", 1036)); // テスト用に任意のポート番号を設定
    
    Config::from(figment) // FigmentからConfigに変換
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client as RocketClient;
    use rocket::http::ContentType;

    #[tokio::test]
    async fn test_update_status_shared() {
        // MongoDBクライアントの初期化
        let mongo_client = MongoClient::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let collection = mongo_client.database("test_db").collection::<Document>("pending_transactions");
        let analytics_collection = MongoClient::with_uri_str("mongodb://localhost:3000")
            .await
            .unwrap()
            .database("analytics_db")
            .collection::<Document>("shared_transactions");

        // テスト用トランザクションを挿入
        let transaction = Transaction {
            sender: "Alice".to_string(),
            receiver: "Bob".to_string(),
            amount: 100.0,
            verifiable_credential: "VerifiableCredential".to_string(),
            signature: "dummy_signature".to_string(),
            transaction_id: "test_tx_id_shared".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            subject: "Test Subject".to_string(),
            action_level: "High".to_string(),
            dimension: "Test Dimension".to_string(),
            fluctuation: "Low".to_string(),
            organism_name: "Test Organism".to_string(),
            details: "Test Details".to_string(),
            goods_or_money: "Money".to_string(),
            transaction_type: "send".to_string(),
            sender_municipality: "Asia-Tokyo".to_string(),
            receiver_municipality: "Europe-London".to_string(),
            sender_continent: "Asia".to_string(),
            receiver_continent: "Europe".to_string(),
            status: "send_pending".to_string(),
            created_at: Utc::now(),
            sender_municipal_id: "sender_municipal_id".to_string(),  // 追加
            receiver_municipal_id: "receiver_municipal_id".to_string(),  // 追加
        };

        let doc = doc! {
            "transaction_id": &transaction.transaction_id,
            "sender": &transaction.sender,
            "receiver": &transaction.receiver,
            "amount": &transaction.amount,
            "verifiable_credential": &transaction.verifiable_credential,
            "signature": &transaction.signature,
            "timestamp": &transaction.timestamp,
            "subject": &transaction.subject,
            "action_level": &transaction.action_level,
            "dimension": &transaction.dimension,
            "fluctuation": &transaction.fluctuation,
            "organism_name": &transaction.organism_name,
            "details": &transaction.details,
            "goods_or_money": &transaction.goods_or_money,
            "transaction_type": &transaction.transaction_type,
            "sender_municipality": &transaction.sender_municipality,
            "receiver_municipality": &transaction.receiver_municipality,
            "sender_continent": &transaction.sender_continent,
            "receiver_continent": &transaction.receiver_continent,
            "status": &transaction.status,
            "created_at": BsonDateTime::from_millis(transaction.created_at.timestamp_millis()),
            "sender_municipal_id": &transaction.sender_municipal_id,  // 追加
            "receiver_municipal_id": &transaction.receiver_municipal_id,  // 追加
        };

        collection.insert_one(doc, None).await.unwrap();

        // Rocketインスタンスの構築
        let rocket = rocket::build()
            .manage(Client::new())
            .manage(Arc::new(Mutex::new(AppState {
                pending_transactions: Arc::new(Mutex::new(HashMap::new())),
                other_continental_chains: Arc::new(Mutex::new(vec![])), 
                mongo_collection: collection.clone(),
                blockchain: Arc::new(Mutex::new(Vec::new())),
                block_collection: mongo_client.database("test_db").collection::<Document>("blocks"), // block_collectionを追加
                municipal_chain_urls: HashMap::new(), // 必要な municipal_chain_urls を追加
            })))
            .mount("/", routes![update_status]);

        let client = RocketClient::untracked(rocket).await.unwrap();

        // ステータス更新リクエスト
        let update_request = serde_json::json!({
            "transaction_id": "test_tx_id_shared",
            "new_status": "shared"
        });

        let response = client.post("/update_status")
            .header(ContentType::JSON)
            .body(update_request.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        // 分析用データベースに移行されたことを確認
        let migrated_doc = analytics_collection.find_one(doc! { "transaction_id": "test_tx_id_shared" }, None).await.unwrap();
        assert!(migrated_doc.is_some());

        // オペレーショナルデータベースから削除されたことを確認
        let deleted_doc = collection.find_one(doc! { "transaction_id": "test_tx_id_shared" }, None).await.unwrap();
        assert!(deleted_doc.is_none());
    }

    #[tokio::test]
    async fn test_create_transaction() {
        // MongoDBクライアントの初期化
        let mongo_client = MongoClient::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("Failed to initialize MongoDB client.");

        // テスト用のコレクションを指定
        let mongo_collection = mongo_client
            .database("test_db")
            .collection::<Document>("pending_transactions");

        // ブロックのコレクションを指定
        let block_collection = mongo_client
            .database("test_db")
            .collection::<Document>("blocks");

        // `municipal_chain_urls`の初期化
        let municipal_chain_urls = HashMap::new(); // テストのために空のHashMapを指定

        // Rocketインスタンスの構築
        let rocket = rocket::build()
            .manage(Client::new())
            .manage(Arc::new(Mutex::new(AppState {
                pending_transactions: Arc::new(Mutex::new(HashMap::new())),
                other_continental_chains: Arc::new(Mutex::new(vec![])),
                mongo_collection,
                block_collection, // block_collection を追加
                blockchain: Arc::new(Mutex::new(Vec::new())),
                municipal_chain_urls, // municipal_chain_urls を追加
            })))
            .mount("/", routes![create_transaction, update_status]);
    
        let client = RocketClient::untracked(rocket).await.unwrap();
    
        // テスト用トランザクションデータ
        let transaction = serde_json::json!({
            "sender": "Alice",
            "receiver": "Bob",
            "amount": 100.0,
            "verifiable_credential": "VerifiableCredential",
            "signature": "dummy_signature",
            "transaction_id": "test_tx_id",
            "timestamp": "2024-01-01T00:00:00Z",
            "subject": "Test Subject",
            "action_level": "High",
            "dimension": "Test Dimension",
            "fluctuation": "Low",
            "organism_name": "Test Organism",
            "details": "Test Details",
            "goods_or_money": "Money",
            "transaction_type": "send",
            "sender_municipality": "Asia-Tokyo",
            "receiver_municipality": "Europe-London",
            "sender_continent": "Asia",
            "receiver_continent": "Europe",
            "sender_municipal_id": "sender_municipal_id",  // 追加
            "receiver_municipal_id": "receiver_municipal_id",  // 追加
        });
    
        let response = client.post("/transaction")
            .header(ContentType::JSON)
            .body(transaction.to_string())
            .dispatch()
            .await;
    
        assert_eq!(response.status(), Status::Accepted);
    }

    #[tokio::test]
    async fn test_update_status() {
        // MongoDBクライアントの初期化
        let mongo_client = MongoClient::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let collection = mongo_client.database("test_db").collection::<Document>("pending_transactions");

        // テスト用トランザクションを挿入
        let transaction = Transaction {
            sender: "Alice".to_string(),
            receiver: "Bob".to_string(),
            amount: 100.0,
            verifiable_credential: "VerifiableCredential".to_string(),
            signature: "dummy_signature".to_string(),
            transaction_id: "test_tx_id".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            subject: "Test Subject".to_string(),
            action_level: "High".to_string(),
            dimension: "Test Dimension".to_string(),
            fluctuation: "Low".to_string(),
            organism_name: "Test Organism".to_string(),
            details: "Test Details".to_string(),
            goods_or_money: "Money".to_string(),
            transaction_type: "send".to_string(),
            sender_municipality: "Asia-Tokyo".to_string(),
            receiver_municipality: "Europe-London".to_string(),
            sender_continent: "Asia".to_string(),
            receiver_continent: "Europe".to_string(),
            status: "send_pending".to_string(),
            created_at: Utc::now(),
            sender_municipal_id: "sender_municipal_id".to_string(),  // 追加
            receiver_municipal_id: "receiver_municipal_id".to_string(),  // 追加
        };

        let doc = doc! {
            "transaction_id": &transaction.transaction_id,
            "sender": &transaction.sender,
            "receiver": &transaction.receiver,
            "amount": &transaction.amount,
            "verifiable_credential": &transaction.verifiable_credential,
            "signature": &transaction.signature,
            "timestamp": &transaction.timestamp,
            "subject": &transaction.subject,
            "action_level": &transaction.action_level,
            "dimension": &transaction.dimension,
            "fluctuation": &transaction.fluctuation,
            "organism_name": &transaction.organism_name,
            "details": &transaction.details,
            "goods_or_money": &transaction.goods_or_money,
            "transaction_type": &transaction.transaction_type,
            "sender_municipality": &transaction.sender_municipality,
            "receiver_municipality": &transaction.receiver_municipality,
            "sender_continent": &transaction.sender_continent,
            "receiver_continent": &transaction.receiver_continent,
            "status": &transaction.status,
            "created_at": BsonDateTime::from_millis(transaction.created_at.timestamp_millis()), 
            "sender_municipal_id": &transaction.sender_municipal_id, // 追加
            "receiver_municipal_id": &transaction.receiver_municipal_id, // 追加
        };

        collection.insert_one(doc, None).await.unwrap();

        // municipalities_dataの読み込み
        let municipalities_data = load_municipalities_data().expect("Failed to load municipalities data");

        // "Default" の設定を取得
        let default_config = municipalities_data.get("Default").expect("Default configuration not found");

        // Rocketインスタンスの構築
        let rocket = rocket::build()
            .manage(Client::new())
            .manage(Arc::new(Mutex::new(AppState {
                pending_transactions: Arc::new(Mutex::new(HashMap::new())),
                other_continental_chains: Arc::new(Mutex::new(vec![])), 
                mongo_collection: collection.clone(),
                block_collection: mongo_client.database("test_db").collection::<Document>("blocks"),
                blockchain: Arc::new(Mutex::new(Vec::new())),
                municipal_chain_urls: default_config.cities
                    .iter()
                    .filter_map(|city| {
                        Some((city.name.clone(), format!("http://localhost:{}", city.city_flask_port)))
                    })
                    .collect(),
            })))
            .mount("/", routes![update_status]);

        let client = RocketClient::untracked(rocket).await.unwrap();

        // ステータス更新リクエスト
        let update_request = serde_json::json!({
            "transaction_id": "test_tx_id",
            "new_status": "send_complete"
        });

        let response = client.post("/update_status")
            .header(ContentType::JSON)
            .body(update_request.to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        // MongoDBでステータスが更新されたことを確認
        let updated_doc = collection.find_one(doc! { "transaction_id": "test_tx_id" }, None).await.unwrap().unwrap();
        assert_eq!(updated_doc.get_str("status").unwrap(), "send_complete");
    }
}
