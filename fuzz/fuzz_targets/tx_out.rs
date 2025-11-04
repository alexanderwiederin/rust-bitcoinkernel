#![no_main]
use bitcoinkernel::{prelude::*, ScriptPubkey, TxOut};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The tx_out corpus from Bitcoin Core contains serialized CTxOut
    // Format: <varint: script_len><script_bytes><8 bytes: amount>

    // Need at least 9 bytes (1 byte varint + 8 bytes amount)
    if data.len() < 9 {
        return;
    }

    // Parse the amount (last 8 bytes)
    let amount = i64::from_le_bytes(data[data.len() - 8..].try_into().unwrap());

    // Everything before the last 8 bytes should be the serialized script
    // (including the varint length prefix)
    let script_data = &data[..data.len() - 8];

    // Skip the varint length prefix and get just the script bytes
    // For simplicity, try creating the script from the data
    let Ok(script) = ScriptPubkey::new(script_data) else {
        return;
    };

    // Create TxOut and verify
    let txout = TxOut::new(&script, amount);
    assert_eq!(txout.value(), amount);

    // Test ref/owned conversions
    let txout_ref = txout.as_ref();
    let owned = txout_ref.to_owned();
    assert_eq!(owned.value(), amount);

    // Test cloning
    let cloned = txout.clone();
    assert_eq!(cloned.value(), amount);
});
