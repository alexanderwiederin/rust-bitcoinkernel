//! Core Bitcoin data structures and operations
//!
//! This module contains the fundamental Bitcoin types like blocks, transactions,
//! scripts, and their associated operations.

pub mod block;
pub mod script;
pub mod transaction;
pub mod verify;

pub use block::{Block, BlockHash, BlockSpentOutputs, BlockTreeEntry, TransactionSpentOutputs};
pub use script::ScriptPubkey;
pub use transaction::{Transaction, TxOut};

pub use block::{BlockSpentOutputsExt, CoinExt, TransactionSpentOutputsExt};
pub use script::ScriptPubkeyExt;
pub use transaction::{TransactionExt, TxOutExt};

pub use verify::{verify, ScriptVerifyError, ScriptVerifyStatus};

pub mod verify_flags {
    pub use super::verify::{
        VERIFY_ALL, VERIFY_ALL_PRE_TAPROOT, VERIFY_CHECKLOCKTIMEVERIFY, VERIFY_CHECKSEQUENCEVERIFY,
        VERIFY_DERSIG, VERIFY_NONE, VERIFY_NULLDUMMY, VERIFY_P2SH, VERIFY_TAPROOT, VERIFY_WITNESS,
    };
}
