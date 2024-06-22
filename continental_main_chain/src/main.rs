#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f64,
}

#[post("/transaction", format = "json", data = "<transaction>")]
async fn create_transaction(transaction: Json<Transaction>) -> Json<Transaction> {
    // トランザクション作成ロジック
    transaction
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![create_transaction])
}
