#[macro_use]
extern crate rocket;

use std::borrow::BorrowMut;
use std::sync::Mutex;

use chain_rs_lib::{Block, Chain};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;

struct BlockChain(Mutex<Chain>);

#[derive(Deserialize)]
struct Payload<'a> {
    value: &'a str,
}

#[get("/blocks")]
fn get_blocks(chain_state: &State<BlockChain>) -> Json<Vec<Block>> {
    let lock = chain_state.0.lock().expect("locked blockchain");
    let blocks = lock.get_blocks();
    Json(blocks)
}

#[post("/blocks", data = "<payload>")]
fn mine_block(payload: Json<Payload>, chain_state: &State<BlockChain>) -> Json<bool> {
    let mut lock = chain_state.0.lock().expect("locked blockchain");
    lock.borrow_mut()
        .add_block(payload.value.to_string())
        .unwrap();
    Json(true)
}

#[launch]
fn rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .manage(BlockChain(Mutex::new(Chain::default())))
        .mount("/", routes![get_blocks, mine_block])
}
