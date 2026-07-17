## CI-enforced
- `check_must_use.sh`: Rust #[must_use] matches header's WARN_UNUSED_RESULT

## Manual
- `check_subtree_kernel_commits.sh`: lists kernel: prefixed commits pulled
  in by the latest libbitcoinkernel-sys/bitcoin subtree merge, for
  reviewing purposes
- `update_lock_files.sh`: regenerates Cargo-minimal.lock (via `cargo
  +nightly update Z minimal-versions`) and Cargo-recent.lock (via `cargo
  update`), for testing against both the crate's minimum and latest supported
  dependency versions
