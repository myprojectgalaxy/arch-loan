use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::Serialize;

/// Running Tests
#[cfg(test)]
mod tests {
    use bitcoincore_rpc::{Auth, Client};
    use borsh::{BorshDeserialize, BorshSerialize};
    use common::constants::*;
    use common::helper::*;
    use sdk::{Pubkey, UtxoMeta};
    use serial_test::serial;
    use std::str::FromStr;

    #[derive(Clone, BorshSerialize, BorshDeserialize)]
    pub struct CounterParams {
        pub value1: u32,
        pub value2: u32,
        pub tx_hex: Vec<u8>,
    }

    #[test]
    #[serial]
    fn back_2_back() {
        start_key_exchange();
        start_dkg();

        println!("...........Single Transactions.................");
        
        let deployed_program_id = Pubkey::from_str(&deploy_program()).unwrap();

        let state_txid = send_utxo();
        read_utxo(NODE1_ADDRESS, format!("{}:1", state_txid.clone()))
            .expect("read utxo should not fail");

        let instruction_data = CounterParams {
            value1: 5,
            value2: 10,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let instruction_data =
            borsh::to_vec(&instruction_data).expect("CounterParams should be serializable");

        let (txid, instruction_hash) = sign_and_send_instruction(
            deployed_program_id.clone(),
            vec![UtxoMeta {
                txid: state_txid.clone(),
                vout: 1,
            }],
            instruction_data,
        )
        .expect("signing and sending a transaction should not fail");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
            .expect("get processed transaction should not fail");

       // println!("processed_tx {:?}", processed_tx);

        let state_txid = &processed_tx.bitcoin_txids[&instruction_hash];
        let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
            .expect("read utxo should not fail");

        assert_eq!(
            utxo.data,
            "Number 5 multiply with Number 10 and Result 50".as_bytes().to_vec()
        );

        let instruction_data = CounterParams {
            value1: 2,
            value2: 50,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let instruction_data =
            borsh::to_vec(&instruction_data).expect("CounterParams should be serializable");

        let (txid, instruction_hash) = sign_and_send_instruction(
            deployed_program_id.clone(),
            vec![UtxoMeta {
                txid: state_txid.clone(),
                vout: 0,
            }],
            instruction_data,
        )
        .expect("signing and sending a transaction should not fail");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
            .expect("get processed transaction should not fail");

        //println!("processed_tx {:?}", processed_tx);

        let state_txid = &processed_tx.bitcoin_txids[&instruction_hash];

        let utxo = read_utxo(
            NODE1_ADDRESS,
            format!("{}:0", state_txid.clone()),
        )
        .expect("read utxo should not fail");

        assert_eq!(
            utxo.data,
            "Number 2 multiply with Number 50 and Result 100".as_bytes().to_vec()
        );
    }

    #[test]
    #[serial]
    fn multiple_instruction_tx() {

        println!("...............................................");
        println!("Multiple Transactions....");
        let deployed_program_id = Pubkey::from_str(&deploy_program()).unwrap();

        let first_state_txid = send_utxo();
        read_utxo(NODE1_ADDRESS, format!("{}:1", first_state_txid.clone()))
            .expect("read utxo should not fail");

        let second_state_txid = send_utxo();
        read_utxo(NODE1_ADDRESS, format!("{}:1", second_state_txid.clone()))
            .expect("read utxo should not fail");

        let instruction_data = CounterParams {
            value1: 20,
            value2: 50,
            tx_hex: hex::decode(prepare_fees()).unwrap(),
        };
        let instruction_data =
            borsh::to_vec(&instruction_data).expect("CounterParams should be serializable");

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
        )
        .expect("signing and sending transaction should not fail");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid)
            .expect("get processed transaction should not fail");
       
       // println!("processed_tx {:?}", processed_tx);

        let state_txid = &processed_tx.bitcoin_txids[&instruction_hash];

        let utxo = read_utxo(NODE1_ADDRESS, format!("{}:0", state_txid.clone()))
            .expect("read utxo should not fail");
        assert_eq!(
            utxo.data,
            "Number 20 multiply with Number 50 and Result 1000".as_bytes().to_vec()
        );

        let utxo = read_utxo(NODE1_ADDRESS, format!("{}:1", state_txid.clone()))
            .expect("read utxo should not fail");
        assert_eq!(
            utxo.data,
            "Number 20 multiply with Number 50 and Result 1000".as_bytes().to_vec()
        );
    }
}
