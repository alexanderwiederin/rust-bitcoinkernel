#![no_main]

use bitcoinkernel::Block;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(block) = Block::try_from(data) {
        let serialized: Vec<u8> = block
            .consensus_encode()
            .expect("Valid block should serialize");

        let reparsed =
            Block::try_from(serialized.as_slice()).expect("Serialized block should deserialize");

        let reserialized: Vec<u8> = reparsed
            .consensus_encode()
            .expect("Reparsed block should serialize");

        assert_eq!(serialized, reserialized, "Non-deterministic serialization");
    }
});
