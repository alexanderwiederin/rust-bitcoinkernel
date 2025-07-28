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

    let status = reader.ibd_status();

    match status {
        IBDStatus::Synced => info!("Bitcoin core is synced"),
        IBDStatus::InIBD => info!("Bitcoin core is in IBD"),
        IBDStatus::NoData => info!("Bitcoin core has no data"),
    }

    let mut block_index = reader.best_validated_block().unwrap();
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

    let block = block_index.block().unwrap();

    info!("Block Hash: {}", block.hash());

    reader.refresh();

    let count = block.transaction_count();

    info!("Transaction count: {}", count);

    let transaction = block.transaction(1).unwrap();
    info!("transaction: {:?}", transaction);
    info!("tx id: {}", transaction.hash().display_order());

    let count = transaction.input_count();
    info!("count: {}", count);

    let input = transaction.input(0).unwrap();
    info!("input: {:?}", input);

    let out_point = input.out_point();
    info!("outpoint: {:?}", out_point);

    info!("outpoint tx id: {:?}", out_point.tx_id());
    info!("outpoint index: {:?}", out_point.index());

    let script = input.script_sig();
    info!("script: {:?}", script);

    info!("script is push only: {}", script.is_push_only());
    info!("script is empty:{}", script.is_empty());

    let n_sequence = input.n_sequence();
    info!("n_sequence: {}", n_sequence);

    let witness = input.witness();
    info!("witness is null: {}", witness.is_null());

    let stack_size = witness.stack_size();
    info!("witness stack size: {}", stack_size);

    let stack_item = witness.stack_item(0).unwrap();
    info!("stack item: {:?}", stack_item);

    let output_count = transaction.output_count();
    info!("output count: {}", output_count);

    let output = transaction.output(0).unwrap();
    info!("output: {:?}", output);

    let value = output.value();
    info!("value: {}", value);

    let is_null = transaction.is_null();
    info!("tx is null: {}", is_null);

    let witness_hash = transaction.witness_hash();
    info!("witness hash: {}", witness_hash);

    let value_out = transaction.value_out();
    info!("value out: {}", value_out);

    let total_size = transaction.total_size();
    info!("total size: {}", total_size);

    let is_coinbase = transaction.is_coinbase();
    info!("is_coinbase: {}", is_coinbase);

    let has_witness = transaction.has_witness();
    info!("has witness: {}", has_witness);

    let script_pubkey = output.script_pubkey();
    info!("script pub key: {:?}", script_pubkey.as_bytes());

    let undo_data = block_index.block_undo().unwrap();
    info!("found undo: {:?}", undo_data);

    let undo_size = undo_data.transaction_count();
    info!("undo size: {}", undo_size);

    for i in 0..undo_data.transaction_count() {
        let transaction_undo_size = undo_data.transaction_undo_size(i);
        info!("transaction undo size: {}", transaction_undo_size);

        let prevout = undo_data.prevout_by_index(i, 0).unwrap();
        info!("prevout: {:?}", prevout);

        info!("value: {}", prevout.value());

        info!("script pubkey: {:?}", prevout.script_pubkey());
    }

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
