use std::env;

use bitcoinkernel::{blockreader::BlockReaderOptions, prelude::*, BlockReader, ContextBuilder};
use log::{error, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .init();

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <data_dir> <blocks_dir>", args[0]);
        std::process::exit(1);
    }

    let data_dir = &args[1];
    let blocks_dir = &args[2];

    info!("Data dir: {}, Blocks dir: {}", data_dir, blocks_dir);

    let context = ContextBuilder::new()
        .chain_type(bitcoinkernel::ChainType::Signet)
        .build()
        .unwrap();

    let reader_options = BlockReaderOptions::new(&context, data_dir, blocks_dir).unwrap();
    let reader = BlockReader::new(reader_options).unwrap();
    let chain = reader.active_chain();

    let mut last_height = 0;

    for (height, block_index) in chain.iter().enumerate() {
        let block = reader.read_block_data(&block_index).unwrap();

        let block_spent_outputs = match reader.read_spent_outputs(&block_index) {
            Ok(outputs) => Some(outputs),
            Err(e) => {
                // probably the genesis block, which has no spent outputs
                error!("Failed to read spent outputs for block: {}", e);
                None
            }
        };

        if height % 1000 == 0 {
            info!("height {}, hash: {:?}", height, block.hash());
            match block_spent_outputs {
                Some(spent_outputs) => {
                    info!(
                        "read block spent outputs with count: {}",
                        spent_outputs.count()
                    );
                }
                None => {
                    error!("no spent outputs available - probably the genesis block");
                }
            }
        }

        last_height = height;
    }

    info!("last block height: {}", last_height);

    Ok(())
}
