# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New `btck_ConsensusParams` opaque type for holding consensus parameters
- New `btck_chain_parameters_get_consensus_params` for extracting consensus params from `btck_ChainParameters` (lifetime-bound to the chain parameters object)
- New `btck_block_check` for context-free block validation (size limits, coinbase structure, sigop limits, with optional POW and merkle-root checks via `btck_BlockCheckFlags`)
- New `btck_BlockCheckFlags` bitflag type with `BASE`, `POW`, `MERKLE`, and `ALL` variants
- New `btck_transaction_get_locktime` for retrieving a transaction's `nLockTime`
- New `btck_transaction_input_get_sequence` for retrieving a transaction input's `nSequence`
- New `btck_block_tree_entry_get_ancestor` for retrieving the ancestor of a block tree entry at a given height
- New `btck_block_header_to_bytes` for serializing a block header to its 80-byte consensus encoding
- Named constants for all enum-like types (`btck_BlockValidationResult_*`, `btck_ChainType_*`, `btck_LogCategory_*`, `btck_LogLevel_*`, `btck_ScriptVerificationFlags_*`, `btck_ScriptVerifyStatus_*`, `btck_SynchronizationState_*`, `btck_ValidationMode_*`, `btck_Warning_*`) exported from `libbitcoinkernel-sys`

### Changed
- Replaced bindgen-generated bindings with hand-written FFI bindings
- `btck_chain_get_by_height` `block_height` parameter type changed from `int` to `int32_t`
- `btck_chain_get_height` return type changed from `int` to `int32_t`
- `btck_block_validation_state_destroy` now accepts a null pointer (removed `NONNULL` annotation)

## [0.2.0] - 2026-01-26

### Added
- New `btck_block_tree_entry_equals` function for comparing BlockTreeEntry objects (096924d39d64)
- New `btck_PrecomputedTransactionData` object for holding transaction hashes required when validating scripts (eb0594e23f0c)
- New `btck_BlockHeader` object for holding block headers (9a9d797ef6ed)
- New functions for working with block headers `btck_block_header_get_*`, `btck_block_get_header`, `btck_block_tree_entry_get_block_header` (9a9d797ef6ed)
- New `btck_chainstate_manager_process_block_header` for processing block headers - validating them, and adding them to the chainstate manager's block tree
  data structure (9a9d797ef6ed)
- New `btck_chainstate_manager_get_best_entry` for retrieving the block tree entry with the most known cumulative proof of work (9a9d797ef6ed)
- New `btck_block_validation_state_*` functions for managing an owned block validation state, that may be used for processing block headers (9a9d797ef6ed)

### Changed
- `data_directory` and `blocks_directory` parameters in `btck_chainstate_manager_options_create` now allow null values to represent empty paths (6657bcbdb4d0)
- `btck_script_pubkey_verify` now takes a `btck_PrecomputedTransactionData` instead of an array of outputs for verifying taproot outputs (eb0594e23f0c)

## [0.1.1] - 2025-24-11

### Fixed
- Precise package excludes to ensure the test/fuzz directory is included
  in the packaged crate correctly.
