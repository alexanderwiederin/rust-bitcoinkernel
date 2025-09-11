//! Core Bitcoin data structures and operations
//!
//! This module contains the fundamental Bitcoin types like blocks, transactions,
//! scripts, and their associated operations.

pub mod block;

pub use block::{
    Block, BlockHash, BlockSpentOutputs, BlockTreeEntry, Coin, TransactionSpentOutputs,
};

pub use block::{BlockSpentOutputsExt, CoinExt, TransactionSpentOutputsExt};
