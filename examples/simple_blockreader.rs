use bitcoinkernel::{
    blockreader::{BlockReader, BlockReaderError},
    ChainType, IBDStatus, Log, Logger,
};
use env_logger;
use log::trace;

struct KernelLogger {}

impl Log for KernelLogger {
    fn log(&self, message: &str) {
        trace!("KERNEL: {}", message);
    }
}

fn main() -> Result<(), Box<BlockReaderError>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let _kernel_logger = Logger::new(KernelLogger {}).expect("Failed to create kernel logger");

    let datadir = "/Users/xyz/Library/Application Support/Bitcoin/signet";

    let mut reader = BlockReader::new(datadir, ChainType::SIGNET)?;
    println!("Blockreader created successfully");

    let status = reader.get_ibd_status();

    match status {
        IBDStatus::Synced => println!("Bitcoin core is synced"),
        IBDStatus::InIBD => println!("Bitcoin core is in IBD"),
        IBDStatus::NoData => println!("Bitcoin core has no data"),
    }

    /* let best_block = reader.get_best_validated_block(); */

    /* println!("Best Block height: {}", best_block.unwrap().height()); */
    let headers_data = reader.get_headers_raw(100, 2)?;
    for header in headers_data.chunks_exact(80) {
        println!("HELLO: {:?}", header);
    }

    let genesis_hash = reader.get_genesis_hash()?;
    println!("HASH: {:?}", genesis_hash);

    let has_block_10 = reader.has_block(10);
    println!("Has Block 10: {}", has_block_10);

    let chain_height = reader.get_chain_height();
    println!("chain height: {}", chain_height);

    let block_hash = reader.get_block_hash(chain_height)?;
    println!("block_hash: {:?}", block_hash);

    let block = reader.get_block(chain_height)?;
    println!("block: {:?}", block);

    println!("Have same hash: {}", block.get_hash() == block_hash);

    let best_validated_block = reader.get_best_validated_block().unwrap();
    println!(
        "Best Validated Block has same hash: {}",
        best_validated_block.block_hash() == block_hash
    );

    let block_index_by_hash = reader.get_block_index_by_hash(&block_hash).unwrap();
    println!(
        "Has same height: {}, has same hash: {}",
        block_index_by_hash.height() == chain_height,
        block_index_by_hash.block_hash() == block_hash
    );

    let is_in_active_chain = reader.is_block_in_active_chain(&block_index_by_hash);
    println!("Is in active chain: {}", is_in_active_chain);

    let block_index_by_height = reader.get_block_index_by_height(chain_height).unwrap();
    println!(
        "Block Index by height yields same hash: {}",
        block_index_by_height.block_hash() == block_index_by_hash.block_hash()
    );

    let block_headers = reader.get_block_header(chain_height).unwrap();
    println!("Got block headers: {:?}", block_headers);

    let block_headers_2 = reader.get_headers_raw(chain_height, 1).unwrap();

    println!("Got second block header: {:?}", block_headers_2);

    Ok(())
}
