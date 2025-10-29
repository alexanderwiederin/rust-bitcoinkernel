#![no_main]

use std::sync::Arc;

use libfuzzer_sys::fuzz_target;

use arbitrary::Arbitrary;

use bitcoinkernel::{
    notifications::types::{BlockValidationStateExt, BlockValidationStateRef},
    Block, ChainType, Context, ContextBuilder, ValidationMode,
};
use fuzz_common::{cleanup_dir, create_chainstate_manager, init_logging, ChainstateSetupConfig};

pub fn create_context_with_notifications(chain_type: ChainType) -> std::sync::Arc<Context> {
    Arc::new(
        ContextBuilder::new()
            .chain_type(chain_type)
            .with_block_tip_notification(|_state, _block_index, _verification_progress| {})
            .with_header_tip_notification(|_state, _height, _timestamp, _presync| {})
            .with_progress_notification(|_title, _progress, _resume_possible| {})
            .with_warning_set_notification(|_warning, _message| {})
            .with_warning_unset_notification(|_warning| {})
            .with_flush_error_notification(|_message| {})
            .with_fatal_error_notification(|_message| {})
            .with_block_checked_validation(|_block, state: BlockValidationStateRef<'_>| {
                assert!(state.mode() != ValidationMode::InternalError)
            })
            .build()
            .unwrap(),
    )
}

#[derive(Debug, Arbitrary)]
pub struct ChainstateManagerInput {
    pub setup: ChainstateSetupConfig,
    pub blocks: Vec<Vec<u8>>,
}

fuzz_target!(|data: ChainstateManagerInput| {
    init_logging();
    let context = create_context_with_notifications(data.setup.chain_type.clone().into());
    let data_dir = data.setup.create_fuzz_data_dir("chainstate");

    let Some(chainman) = create_chainstate_manager(&context, &data.setup, &data_dir) else {
        return;
    };

    if chainman.import_blocks().is_err() {
        cleanup_dir(&data_dir);
        return;
    }

    for block_data in data.blocks {
        if let Ok(block) = Block::try_from(block_data.as_slice()) {
            let _ = chainman.process_block(&block);
        }
    }

    drop(chainman);
    cleanup_dir(&data_dir);
});
