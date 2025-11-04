#![no_main]
use bitcoinkernel::ScriptPubkey;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(script) = ScriptPubkey::try_from(data) else {
        return;
    };

    let serialized: Vec<u8> = script.into();
    let roundtrip = ScriptPubkey::try_from(serialized.as_slice())
        .expect("Serialized script should deserialize");
    let reserialized: Vec<u8> = roundtrip.into();

    assert_eq!(
        serialized, reserialized,
        "Serialization must be stable across roundtrips"
    );
});
