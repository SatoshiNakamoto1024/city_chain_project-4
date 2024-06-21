#[macro_use] extern crate rocket;
use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    prev_hash: String,
    hash: String,
}

#[post("/add_block", format = "json", data = "<block>")]
fn add_block(block: Json<Block>) -> &'static str {
    // ブロックの処理と検証
    "Block added to main chain"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![add_block])
}
