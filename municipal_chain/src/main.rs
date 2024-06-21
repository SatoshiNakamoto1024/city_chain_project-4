#[macro_use] extern crate rocket;
use rocket::serde::{json::Json, Deserialize, Serialize};
use reqwest::Client;

#[derive(Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    prev_hash: String,
    hash: String,
}

#[post("/add_block", format = "json", data = "<block>")]
async fn add_block(block: Json<Block>, client: &rocket::State<Client>) -> &'static str {
    // ブロックの処理と検証

    // メインチェーンへのブロック転送
    let main_chain_url = "http://main_chain:8080/add_block";
    let res = client.post(main_chain_url)
                    .json(&*block)
                    .send()
                    .await;

    match res {
        Ok(_) => "Block added and sent to main chain",
        Err(_) => "Failed to send block to main chain",
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Client::new())
        .mount("/", routes![add_block])
}
