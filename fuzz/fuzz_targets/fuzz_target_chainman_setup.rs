#![no_main]
use std::sync::Arc;

use arbitrary::Arbitrary;
use bitcoinkernel::ChainType;
use fuzz_common::{cleanup_dir, create_chainstate_manager, init_logging, ChainstateSetupConfig};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, Arbitrary)]
pub struct SetupInput {
    pub config: ChainstateSetupConfig,
}

pub fn create_simple_context(chain_type: ChainType) -> std::sync::Arc<bitcoinkernel::Context> {
    use bitcoinkernel::ContextBuilder;

    Arc::new(
        ContextBuilder::new()
            .chain_type(chain_type)
            .build()
            .unwrap(),
    )
}

fuzz_target!(|data: SetupInput| {
    init_logging();

    let context = create_simple_context(data.config.chain_type.clone().into());
    let data_dir = data.config.create_fuzz_data_dir("setup");

    let Some(chainman) = create_chainstate_manager(&context, &data.config, &data_dir) else {
        return;
    };

    drop(chainman);
    cleanup_dir(&data_dir);
});
