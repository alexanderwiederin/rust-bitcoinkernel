# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New `btck_block_tree_entry_equals` function for comparing BlockTreeEntry objects (096924d39d64)
- New `btck_PrecomputedTransactionData` object for holding transaction hashes required when validating scripts (eb0594e23f0c)

### Changed
- `data_directory` and `blocks_directory` parameters in `btck_chainstate_manager_options_create` now allow null values to represent empty paths (6657bcbdb4d0)
- `btck_script_pubkey_verify` now takes a `btck_PrecomputedTransactionData` instead of an array of outputs for verifying taproot outputs (eb0594e23f0c)

## [0.1.1] - 2025-24-11

### Fixed
- Precise package excludes to ensure the test/fuzz directory is included
  in the packaged crate correctly.
