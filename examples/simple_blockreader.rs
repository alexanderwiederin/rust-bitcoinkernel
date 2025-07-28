use std::{thread::sleep, time::Duration};

use bitcoinkernel::{
    blockreader::{BlockReader, BlockReaderError},
    ChainType, IBDStatus, Log, Logger,
};
use env_logger;
use log::{info, trace};

struct KernelLogger {}

impl Log for KernelLogger {
    fn log(&self, message: &str) {
        trace!("KERNEL: {}", message);
    }
}

fn main() -> Result<(), Box<BlockReaderError>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let _kernel_logger = Logger::new(KernelLogger {}).expect("Failed to create kernel logger");

    let datadir = "/Users/xyz/Library/Application Support/Bitcoin/signet";

    let reader = BlockReader::new(datadir, ChainType::SIGNET).unwrap();
    info!("Blockreader created successfully");

    let status = reader.get_ibd_status();

    match status {
        IBDStatus::Synced => info!("Bitcoin core is synced"),
        IBDStatus::InIBD => info!("Bitcoin core is in IBD"),
        IBDStatus::NoData => info!("Bitcoin core has no data"),
    }

    let mut block_index = reader.get_best_validated_block().unwrap();
    info!(
        "Got block index: prev hash={:?}",
        block_index.prev_block_hash().unwrap().to_string()
    );
    info!("Hash: {}", block_index.block_hash().display_order());
    info!("Height: {}", block_index.height());
    info!("Transaction count: {}", block_index.transaction_count());
    info!("Block index merkle root: {}", block_index.merkle_root());
    info!("bits: {:08x}", block_index.bits());
    info!("nonce: {}", block_index.nonce());
    info!("median time past: {}", block_index.median_time_past());
    info!("has block data: {}", block_index.has_block_data());
    info!("has undo data: {}", block_index.has_undo_data());
    info!(
        "has valid transactions: {}",
        block_index.has_valid_transactions()
    );
    info!("has valid chain: {}", block_index.has_valid_chain());
    info!("has valid scripts: {}", block_index.has_valid_scripts());
    info!("has failed: {}", block_index.is_failed());
    info!("has witness: {}", block_index.has_witness());

    let block = block_index.get_block().unwrap();

    info!("Block Hash: {}", block.get_hash());

    reader.refresh();

    let count = block.get_transaction_count();

    info!("Transaction count: {}", count);

    let transaction = block.get_transaction(1).unwrap();
    info!("transaction: {:?}", transaction);
    info!("tx id: {}", transaction.get_hash().display_order());

    let count = transaction.get_input_count();
    info!("count: {}", count);

    let input = transaction.get_input(0).unwrap();
    info!("input: {:?}", input);

    let out_point = input.get_out_point();
    info!("outpoint: {:?}", out_point);

    info!("outpoint tx id: {:?}", out_point.get_tx_id());
    info!("outpoint index: {:?}", out_point.get_index());

    let script = input.get_script_sig();
    info!("script: {:?}", script);

    info!("script is push only: {}", script.is_push_only());
    info!("script is empty:{}", script.is_empty());

    let n_sequence = input.get_n_sequence();
    info!("n_sequence: {}", n_sequence);

    let witness = input.get_witness();
    info!("witness is null: {}", witness.is_null());

    let stack_size = witness.get_stack_size();
    info!("witness stack size: {}", stack_size);

    let stack_item = witness.get_stack_item(0).unwrap();
    info!("stack item: {:?}", stack_item);

    let output_count = transaction.get_output_count();
    info!("output count: {}", output_count);

    let output = transaction.get_output(0).unwrap();
    info!("output: {:?}", output);

    let value = output.get_value();
    info!("value: {}", value);

    let is_null = transaction.is_null();
    info!("tx is null: {}", is_null);

    let witness_hash = transaction.get_witness_hash();
    info!("witness hash: {}", witness_hash);

    let value_out = transaction.get_value_out();
    info!("value out: {}", value_out);

    let total_size = transaction.get_total_size();
    info!("total size: {}", total_size);

    let is_coinbase = transaction.is_coinbase();
    info!("is_coinbase: {}", is_coinbase);

    let has_witness = transaction.has_witness();
    info!("has witness: {}", has_witness);

    let script_pubkey = output.get_script_pubkey();
    info!("script pub key: {:?}", script_pubkey.as_bytes());

    // while let Ok(block_index) = block_index.previous() {
    //     info!("Height: {}", block_index.height());
    //     info!("Hash: {}", block_index.block_hash());
    //
    //     let block = block_index.get_block().unwrap();
    //
    //     for i in 0..block.get_transaction_count() {
    //         let transaction = block.get_transaction(i).unwrap();
    //         info!("transaction {}", i);
    //
    //         for j in 0..transaction.get_input_count() {
    //             let input = transaction.get_input(j).unwrap();
    //             let tx_id = input.get_out_point().get_tx_id();
    //             let index = input.get_out_point().get_index();
    //             info!("input #{}: tx_id: {}, index: {}", j, tx_id, index);
    //         }
    //
    //         for j in 0..transaction.get_output_count() {
    //             let output = transaction.get_output(j).unwrap();
    //             let value = output.get_value();
    //             info!("output #{}: value: {}", j, value);
    //         }
    //     }
    // }

    Ok(())
}
