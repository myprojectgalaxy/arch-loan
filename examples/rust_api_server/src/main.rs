use actix_web::{web, App, HttpServer, Responder, HttpResponse, get,post};
use serde::{Deserialize, Serialize};
use bitcoincore_rpc::{Auth, Client};
use borsh::{BorshSerialize, BorshDeserialize};
use common::constants::*;
use sdk::{Pubkey, UtxoMeta};
use common::helper::*;
use serial_test::serial;
use common::models::*;
use std::thread;
use std::str::FromStr;


// Constants
const NODE1_ADDRESS: &str = "https://bitcoin-node.dev.aws.archnetwork.xyz:18443/wallet/testwallet";
const USER: &str = "bitcoin";
const PASSWORD: &str = "428bae8f3c94f8c39c50757fc89c39bc7e6ebc70ebf8f618";

#[derive(Clone, BorshSerialize, BorshDeserialize, Deserialize)]
struct HelloWorldParams {
    name: String,
    tx_hex: Vec<u8>,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

#[post("/back_2_back")]
async fn back_2_back(params: web::Json<HelloWorldParams>) -> impl Responder {
    let rpc = Client::new(NODE1_ADDRESS, Auth::UserPass(USER.to_string(), PASSWORD.to_string())).unwrap();
    let deployed_program_id = Pubkey::from_str(&deploy_program()).unwrap();
    let state_txid = send_utxo();

    // Simulate the operations in the test
    let instruction_data =
        borsh::to_vec(&params.0).expect("HelloWorldParams should be serializable");

    let (txid, instruction_hash) = sign_and_send_instruction(
        deployed_program_id.clone(),
        vec![UtxoMeta {
            txid: state_txid.clone(),
            vout: 1,
        }],
        instruction_data,
    ).expect("signing and sending a transaction should not fail");

    let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
        .expect("get processed transaction should not fail");

    let state_txid = processed_tx.bitcoin_txids[&instruction_hash].clone();
    let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
        .expect("read utxo should not fail");

    if utxo.data == format!("Hello {}!", params.name).as_bytes().to_vec() {
        HttpResponse::Ok().json(ApiResponse { message: "Success".to_string() })
    } else {
        HttpResponse::InternalServerError().json(ApiResponse { message: "Failed".to_string() })
    }
}

#[post("/multiple_instruction_tx")]
async fn multiple_instruction_tx(params: web::Json<HelloWorldParams>) -> impl Responder {
    start_key_exchange();
    start_dkg();

    let rpc = Client::new(NODE1_ADDRESS, Auth::UserPass(USER.to_string(), PASSWORD.to_string())).unwrap();
    let deployed_program_id = Pubkey::from_str(&deploy_program()).unwrap();

    let first_state_txid = send_utxo();
    read_utxo(NODE1_ADDRESS, format!("{}:1", first_state_txid.clone()))
        .expect("read utxo should not fail");

    let second_state_txid = send_utxo();
    read_utxo(NODE1_ADDRESS, format!("{}:1", second_state_txid.clone()))
        .expect("read utxo should not fail");

    let instruction_data =
        borsh::to_vec(&params.0).expect("HelloWorldParams should be serializable");

    let (txid, instruction_hash) = sign_and_send_instruction(
        deployed_program_id.clone(),
        vec![
            UtxoMeta {
                txid: first_state_txid.clone(),
                vout: 1,
            },
            UtxoMeta {
                txid: second_state_txid.clone(),
                vout: 1,
            },
        ],
        instruction_data,
    ).expect("signing and sending transaction should not fail");

    let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
        .expect("get processed transaction should not fail");

    let state_txid = processed_tx.bitcoin_txids[&instruction_hash].clone();

    let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
        .expect("read utxo should not fail");

    if utxo.data == format!("Hello {}!", params.name).as_bytes().to_vec() {
        HttpResponse::Ok().json(ApiResponse { message: "Success".to_string() })
    } else {
        HttpResponse::InternalServerError().json(ApiResponse { message: "Failed".to_string() })
    }
}

// Handler for the root endpoint
#[get("/hello")]
async fn hello() -> impl Responder {

    HttpResponse::Ok().json(ApiResponse {
        message: "Success".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(back_2_back)
            .service(multiple_instruction_tx)
            .service(hello) 
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
