# libbitcoinkernel-sys

Raw FFI bindings to
[libbitcoinkernel](https://github.com/bitcoin/bitcoin/blob/master/src/kernel/bitcoinkernel.h),
the Bitcoin Core kernel library.

## Warning

This crate is not intended for direct use. Use the safe, idiomatic
[bitcoinkernel](https://crates.io/crates/bitcoinkernel) crate instead.

## Bindings

The bindings are hand-written and manually maintained against the upstream C
header.

For the authoritative API documentation, refer to the upstream C header:
https://github.com/bitcoin/bitcoin/blob/master/src/kernel/bitcoinkernel.h

## Supported targets

Tested on x86_64 Linux, ARM64 Linux, Windows, and macOS via CI.
