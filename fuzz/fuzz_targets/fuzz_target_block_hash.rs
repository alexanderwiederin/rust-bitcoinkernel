#![no_main]

use bitcoinkernel::Block;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(block) = Block::try_from(data) {
        let hash1 = block.hash();
        let hash2 = block.hash();
        assert_eq!(hash1, hash2, "Hash should be deterministic");

        let serialized: Vec<u8> = block.consensus_encode().unwrap();
        let reparsed = Block::try_from(serialized.as_slice()).unwrap();

        let hash_original = block.hash();
        let hash_reparsed = reparsed.hash();
        assert_eq!(
            hash_original, hash_reparsed,
            "Hash must be preserved after round-trip"
        );

        let bytes1: [u8; 32] = (&hash_original).into();
        let bytes2 = hash_original.to_bytes();
        assert_eq!(bytes1, bytes2, "Hash byte conversions should match");

        let hash_clone = block.hash();
        assert_eq!(hash_original, hash_clone);
    }
});
