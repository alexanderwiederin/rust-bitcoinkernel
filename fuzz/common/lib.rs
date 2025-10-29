use arbitrary::Arbitrary;
use bitcoinkernel::{ChainType, ChainstateManager, ChainstateManagerOptions, Context, KernelError};
use std::sync::{Arc, Once};

#[derive(Debug, Clone, Arbitrary)]
pub enum FuzzChainType {
    MAINNET,
    TESTNET,
    REGTEST,
    SIGNET,
}

impl From<FuzzChainType> for ChainType {
    fn from(val: FuzzChainType) -> ChainType {
        match val {
            FuzzChainType::MAINNET => ChainType::Mainnet,
            FuzzChainType::TESTNET => ChainType::Testnet,
            FuzzChainType::REGTEST => ChainType::Regtest,
            FuzzChainType::SIGNET => ChainType::Signet,
        }
    }
}

#[derive(Debug, Clone, Arbitrary)]
pub struct ChainstateSetupConfig {
    pub data_dir: String,
    pub chain_type: FuzzChainType,
    pub wipe_block_index: bool,
    pub wipe_chainstate_index: bool,
    pub block_tree_db_in_memory: bool,
    pub chainstate_db_in_memory: bool,
    pub worker_threads: i32,
}

impl ChainstateSetupConfig {
    /// Sanitize the data directory string for filesystem use
    pub fn sanitize_data_dir(&self) -> String {
        let sanitized: String = self
            .data_dir
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .take(60)
            .collect();

        if sanitized.is_empty() {
            "default".to_string()
        } else {
            sanitized
        }
    }

    pub fn create_fuzz_data_dir(&self, prefix: &str) -> String {
        format!(
            "/tmp/rust_kernel_fuzz_{}/{}",
            prefix,
            self.sanitize_data_dir()
        )
    }

    pub fn create_blocks_dir(&self, data_dir: &str) -> String {
        format!("{}/blocks", data_dir)
    }
}

static INIT: Once = Once::new();

pub fn init_logging() {
    INIT.call_once(|| {
        bitcoinkernel::disable_logging();
    });
}

pub fn create_chainstate_manager(
    context: &Arc<Context>,
    config: &ChainstateSetupConfig,
    data_dir: &str,
) -> Option<ChainstateManager> {
    let blocks_dir = config.create_blocks_dir(data_dir);

    let chainman_opts = match ChainstateManagerOptions::new(context, data_dir, &blocks_dir) {
        Ok(opts) => opts,
        Err(KernelError::CStringCreationFailed(_)) => return None,
        Err(err) => panic!("this should never happen: {}", err),
    }
    .wipe_db(config.wipe_block_index, config.wipe_chainstate_index)
    .block_tree_db_in_memory(config.block_tree_db_in_memory)
    .chainstate_db_in_memory(config.chainstate_db_in_memory)
    .worker_threads(config.worker_threads);

    match ChainstateManager::new(chainman_opts) {
        Err(KernelError::Internal(_)) => None,
        Err(err) => {
            cleanup_dir(data_dir);
            panic!("this should never happen: {}", err);
        }
        Ok(chainman) => Some(chainman),
    }
}

pub fn cleanup_dir(dir: &str) {
    if let Err(e) = std::fs::remove_dir_all(dir) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!("Cleanup failed for {}: {}", dir, e);
        }
    }
}
