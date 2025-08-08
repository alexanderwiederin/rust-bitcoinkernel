use bitcoinkernel::{
    BlockReader, BlockReaderError, BlockReaderIndex, BlockRef, BlockUndoRef, ChainType,
};
use env_logger;
use log::info;
use std::thread;
use std::time::Instant;

fn setup_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .format_module_path(false)
        .format_target(false)
        .init();
}

fn collect_block_range(
    start_index: BlockReaderIndex,
    count: usize,
) -> Result<Vec<BlockReaderIndex>, BlockReaderError> {
    let mut indexes = Vec::with_capacity(count);
    let mut current = start_index;

    for _ in 0..count {
        indexes.push(current.clone());
        current = current.previous()?;
    }

    Ok(indexes)
}

fn analyze_block(
    index: BlockReaderIndex,
    block_num: usize,
) -> Result<BlockAnalysis, BlockReaderError> {
    let block = index.block()?;
    let undo = index.block_undo()?;

    let height = index.height();
    let tx_count = block.transaction_count();
    let total_value = calculate_total_block_value(&block)?;
    let total_fees = calculate_block_fees(&block, &undo)?;
    let has_large_tx = has_large_transaction(&block);

    Ok(BlockAnalysis {
        block_num,
        height,
        tx_count,
        total_value,
        total_fees,
        has_large_tx,
    })
}

#[derive(Debug)]
struct BlockAnalysis {
    block_num: usize,
    height: i32,
    tx_count: usize,
    total_value: i64,
    total_fees: i64,
    has_large_tx: bool,
}

fn parallel_chain_analysis(
    start_index: BlockReaderIndex,
    num_blocks: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Parallel Chain Analysis ===");
    info!("Analyzing {} blocks using multiple threads", num_blocks);

    let start_time = Instant::now();

    let indexes = collect_block_range(start_index, num_blocks)?;
    info!("Collected {} block indexes", indexes.len());

    let num_threads = std::cmp::min(4, num_blocks);
    let chunk_size = (indexes.len() + num_threads - 1) / num_threads;

    info!(
        "Using {} threads with ~{} blocks per thread",
        num_threads, chunk_size
    );

    let mut handles = Vec::new();

    for (thread_id, chunk) in indexes.chunks(chunk_size).enumerate() {
        let chunk = chunk.to_vec();

        let handle = thread::spawn(move || {
            info!("Thread {} starting with {} blocks", thread_id, chunk.len());
            let mut results = Vec::new();

            for (i, index) in chunk.into_iter().enumerate() {
                match analyze_block(index, i + thread_id * chunk_size) {
                    Ok(analysis) => {
                        info!(
                            "Thread {} completed block {} (height {})",
                            thread_id, analysis.block_num, analysis.height
                        );
                        results.push(analysis);
                    }
                    Err(e) => {
                        log::error!("Thread {} failed on block {}: {}", thread_id, i, e);
                    }
                }
            }

            info!(
                "Thread {} finished, analyzed {} blocks",
                thread_id,
                results.len()
            );
            results
        });

        handles.push(handle);
    }

    let mut all_results = Vec::new();
    for (thread_id, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(thread_results) => {
                info!("Thread {} joined successfully", thread_id);
                all_results.extend(thread_results);
            }
            Err(_) => {
                log::error!("Thread {} panicked!", thread_id);
            }
        }
    }

    let elapsed = start_time.elapsed();

    info!("=== Parallel Analysis Complete ===");
    info!("Processed {} blocks in {:?}", all_results.len(), elapsed);
    info!(
        "Average: {:.2} blocks/second",
        all_results.len() as f64 / elapsed.as_secs_f64()
    );

    all_results.sort_by_key(|a| a.block_num);

    let total_transactions: usize = all_results.iter().map(|a| a.tx_count).sum();
    let total_fees: i64 = all_results.iter().map(|a| a.total_fees).sum();
    let large_tx_blocks = all_results.iter().filter(|a| a.has_large_tx).count();

    info!("📊 Summary Statistics:");
    info!("  Total transactions: {}", total_transactions);
    info!("  Total fees: {} BTC", satoshis_to_btc(total_fees));
    info!("  Blocks with large transactions: {}", large_tx_blocks);
    info!(
        "  Average transactions per block: {:.1}",
        total_transactions as f64 / all_results.len() as f64
    );

    Ok(())
}

fn sequential_chain_analysis(
    start_index: BlockReaderIndex,
    num_blocks: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Sequential Chain Analysis (for comparison) ===");

    let start_time = Instant::now();
    let mut current = start_index;

    for i in 0..num_blocks {
        let analysis = analyze_block(current.clone(), i)?;
        info!(
            "Sequential: Block {} (height {}) complete",
            analysis.block_num, analysis.height
        );
        current = current.previous()?;
    }

    let elapsed = start_time.elapsed();
    info!("Sequential analysis took {:?}", elapsed);
    info!(
        "Average: {:.2} blocks/second",
        num_blocks as f64 / elapsed.as_secs_f64()
    );

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
                if let Ok(prevout) = undo.prevout_by_index(undo_tx_idx, prevout_idx) {
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

    let reader = BlockReader::new(
        "/Users/xyz/Library/Application Support/Bitcoin/signet",
        ChainType::SIGNET,
    )?;

    let start_index = reader
        .best_validated_block_index()
        .ok_or("No validated blocks found")?;

    info!(
        "🚀 Starting multithreaded block analysis from height: {}",
        start_index.height()
    );

    let num_blocks = 100;

    parallel_chain_analysis(start_index.clone(), num_blocks)?;

    info!("\n");
    sequential_chain_analysis(start_index, num_blocks)?;

    info!("✅ Multithreaded analysis complete!");

    Ok(())
}
