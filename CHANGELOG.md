# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added a `PrecomputedTransactionData` struct for holding transaction hashes required for script verification. It may be initialized with and without an array of outputs spent by the transaction. If no outputs are passed, no taproot verification is possible.
- Added a `BlockHeader` struct for holding block header data. This may be used to retrieve block header struct internals, and to process block headers for the purpose of "headers-first" synchronization. Also add a `ChainstateManager` method to process block headers, i.e. validate and add them to its internal block tree data structure.

### Changed
- Updated to latest libbitcoinkernel-sys with btck_PrecomputedTransactionData` changes.
- The `verify` function now takes a `PrecomputedTransactionData` instead of an array of outputs spent by the transaction. The user is now always required to pass this struct to the function. This is done to encourage its use and protect against quadratic hashing costs.

## [0.1.1] - 2025-24-11

### Fixed
- Updated to latest libbitcoinkernel-sys with cmake packaging include fix.
