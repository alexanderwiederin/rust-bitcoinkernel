use bitcoinkernel::{BlockReader, ChainType};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    let reader = BlockReader::new(
        "/Users/xyz/Library/Application Support/Bitcoin/signet",
        ChainType::SIGNET,
    )?;

    Ok(())
}
