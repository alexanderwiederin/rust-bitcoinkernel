use std::{env, process};

use bitcoinkernel::{
    BlockReader, BlockReaderError, BlockReaderIndex, BlockRef, BlockUndoRef, ChainType,
};
use env_logger;
use log::info;

fn setup_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .format_module_path(false)
        .format_target(false)
        .init();
}

fn analyze_chain(start_index: BlockReaderIndex) -> Result<(), BlockReaderError> {
    info!("=== Chain Analysis Example ===");

    let mut current = start_index;
    let mut blocks_analyzed = 0;

    while blocks_analyzed < 5 {
        let block = current.block()?;

        info!("Block {}: Height {}", blocks_analyzed + 1, current.height());

        let tx_count = block.transaction_count();
        let total_value = calculate_total_block_value(&block)?;

        info!(
            "{} transactions, {} BTC total",
            tx_count,
            satoshis_to_btc(total_value),
        );

        if tx_count > 3000 {
            info!("High activity block!");
        }

        if has_large_transaction(&block) {
            info!("Contains large transaction (>10 BTC)");
        }

        match current.previous() {
            Some(prev) => current = prev,
            None => {
                info!("Reached genesis block, stopping analysis");
                break;
            }
        }
        blocks_analyzed += 1;
    }

    Ok(())
}

fn compare_adjacent_blocks(index: &BlockReaderIndex) -> Result<(), BlockReaderError> {
    info!("=== Comparing Adjacent Blocks ===");

    let current_block = index.block()?;

    let prev_index = match index.previous() {
        Some(prev) => prev,
        None => {
            info!("No previous block to compare (genesis block?)");
            return Ok(());
        }
    };

    let prev_block = prev_index.block()?;

    let current_fees = calculate_block_fees(&current_block, &index.block_undo()?)?;
    let prev_fees = calculate_block_fees(&prev_block, &prev_index.block_undo()?)?;

    info!(
        "Current block ({}): {} transactions, {} sats fees",
        index.height(),
        current_block.transaction_count(),
        current_fees
    );

    info!(
        "Previous block ({}): {} transactions, {} sats fees",
        prev_index.height(),
        prev_block.transaction_count(),
        prev_fees
    );

    if current_fees > prev_fees * 2 {
        info!(
            "Fee spike detected! {}% increase",
            ((current_fees - prev_fees) * 100 / prev_fees)
        );
    }

    Ok(())
}

fn calculate_total_block_value(block: &BlockRef) -> Result<i64, BlockReaderError> {
    let mut total = 0i64;

    for tx_idx in 0..block.transaction_count() {
        if let Some(tx) = block.transaction(tx_idx) {
            total += tx.value_out();
        }
    }

    Ok(total)
}

fn calculate_block_fees(block: &BlockRef, undo: &BlockUndoRef) -> Result<i64, BlockReaderError> {
    let mut total_fees = 0i64;

    for tx_idx in 1..block.transaction_count() {
        if let Some(tx) = block.transaction(tx_idx) {
            let undo_tx_idx = (tx_idx - 1) as u64;
            let undo_size = undo.transaction_undo_size(undo_tx_idx);

            let mut inputs_value = 0i64;
            for prevout_idx in 0..undo_size {
                if let Some(prevout) = undo.prevout_by_index(undo_tx_idx, prevout_idx) {
                    inputs_value += prevout.value();
                }
            }

            let fee = inputs_value - tx.value_out();
            if fee >= 0 {
                total_fees += fee;
            }
        }
    }

    Ok(total_fees)
}

fn has_large_transaction(block: &BlockRef) -> bool {
    for tx_idx in 0..block.transaction_count() {
        if let Some(tx) = block.transaction(tx_idx) {
            if tx.value_out() > 1_000_000_000 {
                return true;
            }
        }
    }
    false
}

fn satoshis_to_btc(sats: i64) -> f64 {
    sats as f64 / 100_000_000.0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_data_dir>", args[0]);
        process::exit(1);
    }

    let data_dir = args[1].clone();

    let reader = BlockReader::new(&data_dir, ChainType::SIGNET).unwrap();

    let start_index = reader
        .best_validated_block_index()
        .ok_or("No validated blocks found")?;

    info!("Starting from block height: {}", start_index.height());

    analyze_chain(start_index.clone())?;

    compare_adjacent_blocks(&start_index)?;

    Ok(())
}
