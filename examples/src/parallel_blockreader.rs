use bitcoinkernel::{
    BlockReader, BlockReaderError, BlockReaderIndex, BlockRef, BlockUndoRef, ChainType,
};
use env_logger;
use log::info;
use std::time::Instant;
use std::{env, process, thread};

fn setup_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .format_module_path(false)
        .format_target(false)
        .init();
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
    info!("=== Parallel Chain Analysis (Forward) ===");
    info!("Analyzing {} blocks using multiple threads", num_blocks);

    let start_time = Instant::now();

    let indexes = start_index
        .iter_forwards()
        .take(num_blocks)
        .collect::<Vec<_>>();
    info!("Collected {} block indexes", indexes.len());

    let num_threads = std::cmp::min(4, indexes.len());
    let chunk_size = indexes.len().div_ceil(num_threads);

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

    info!("Summary Statistics:");
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
    info!("=== Sequential Chain Analysis (Forward, for comparison) ===");

    let start_time = Instant::now();
    let iterator = start_index.iter_forwards();

    for (i, current) in iterator.take(num_blocks).enumerate() {
        let analysis = analyze_block(current, i)?;
        info!(
            "Sequential: Block {} (height {}) complete",
            analysis.block_num, analysis.height
        );
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

    let best_index = reader
        .best_validated_block_index()
        .ok_or("No validated blocks found")?;

    let num_blocks = 100;

    let start_height = best_index.height().saturating_sub(num_blocks);
    let start_index = reader
        .block_index_at(start_height)
        .ok_or("Could not find starting block index")?;

    info!(
        "Starting multithreaded block analysis (forward) from height: {}",
        start_index.height()
    );

    parallel_chain_analysis(start_index.clone(), num_blocks as usize)?;

    info!("\n");
    sequential_chain_analysis(start_index, num_blocks as usize)?;

    info!("Multithreaded analysis complete!");

    Ok(())
}
