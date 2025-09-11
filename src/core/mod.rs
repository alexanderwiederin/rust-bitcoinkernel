//! Core Bitcoin data structures and operations
//!
//! This module contains the fundamental Bitcoin types like blocks, transactions,
//! scripts, and their associated operations.

pub mod block;
pub mod transaction;

pub use block::{
    Block, BlockHash, BlockSpentOutputs, BlockTreeEntry, Coin, TransactionSpentOutputs,
};
pub use transaction::{Transaction, TxOut};

pub use block::{BlockSpentOutputsExt, CoinExt, TransactionSpentOutputsExt};
pub use transaction::{TransactionExt, TxOutExt};
