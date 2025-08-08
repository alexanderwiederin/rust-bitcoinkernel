use bitcoinkernel::{BlockReader, BlockReaderIndex, ChainType};
use env_logger;
use log::{info, warn};

fn setup_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .format_module_path(false)
        .format_target(false)
        .init();
}

fn process_block(index: &BlockReaderIndex, block_number: usize) {
    info!("=== Block {} ===", block_number);
    info!(
        "Height: {} - Hash: {}",
        index.height(),
        index.block_hash().display_order()
    );

    let block = index.block().unwrap();
    let block_undo = index.block_undo().unwrap();

    info!("Transactions: {}", block.transaction_count());

    if let Some(coinbase_tx) = block.transaction(0) {
        info!("Coinbase value: {} satoshis", coinbase_tx.value_out());
        info!("Coinbase hash: {}", coinbase_tx.hash().display_order());
    }

    info!("Undo transactions: {}", block_undo.transaction_count());

    let mut total_fees = 0i64;
    for tx_idx in 1..block.transaction_count() {
        if let Some(tx) = block.transaction(tx_idx) {
            let undo_tx_idx = (tx_idx - 1) as u64;
            let undo_size = block_undo.transaction_undo_size(undo_tx_idx);

            let mut inputs_value = 0i64;
            for prevout_idx in 0..undo_size {
                if let Ok(prevout) = block_undo.prevout_by_index(undo_tx_idx, prevout_idx) {
                    inputs_value += prevout.value();
                }
            }

            let fee = inputs_value - tx.value_out();
            if fee >= 0 {
                total_fees += fee;
            } else {
                warn!("Negative fee detected for tx {}: {} sats", tx_idx, fee);
            }
        }
    }

    info!("Total fees: {} satoshis", total_fees);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    info!("Initializing BlockReader...");
    let reader = BlockReader::new(
        "/Users/xyz/Library/Application Support/Bitcoin/signet",
        ChainType::SIGNET,
    )
    .unwrap();

    let mut current_index = reader.best_validated_block_index().unwrap();

    for i in 0..10 {
        process_block(&current_index, i + 1);

        match current_index.previous() {
            Ok(prev) => current_index = prev,
            Err(e) => {
                warn!("Reached end of chain or error at block {}: {}", i + 1, e);
                break;
            }
        }
    }

    info!("\n");
    info!("======  Starting iterator approach  ======");
    info!("\n");

    let start_index = reader.best_validated_block_index().unwrap();

    for (i, block_index) in start_index.iter_backwards().take(10).enumerate() {
        process_block(&block_index, i);
    }

    info!("\n");
    info!("======  Starting while iterator approach  ======");
    info!("\n");

    let start_index_2 = reader.best_validated_block_index().unwrap();

    let index_200_000 = reader.block_index_at(200_000).unwrap();
    let block_hash = index_200_000.block_hash();

    for block_index in start_index_2.iter_backwards_while(move |idx| idx.block_hash() != block_hash)
    {
        info!("block height: {}", block_index.height());
    }

    // Iterator Integration demo

    let best_block_index = reader.best_validated_block_index().unwrap();

    let _ = best_block_index
        .iter_backwards()
        .take(100)
        .filter(|idx| idx.has_block_data())
        .map(|idx| idx.block().unwrap())
        .filter(|block| block.transaction_count() > 1000)
        .collect::<Vec<_>>();

    Ok(())
}
