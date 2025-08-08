use log::info;
use std::env;
use std::fmt;
use std::process;

use bitcoin::hashes::Hash;
use bitcoin::{PrivateKey, XOnlyPublicKey};
use bitcoinkernel::BlockReader;
use bitcoinkernel::BlockReaderIndex;
use bitcoinkernel::ChainType;
use log::error;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use silentpayments::receiving::{Label, Receiver};
use silentpayments::utils::receiving::{
    calculate_shared_secret, calculate_tweak_data, get_pubkey_from_input,
};

fn setup_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .format_module_path(false)
        .format_target(false)
        .init();
}

fn vec_to_hex_string(data: &Vec<u8>) -> String {
    let mut hex_string = String::with_capacity(data.len() * 2);
    for byte in data {
        hex_string.push_str(&format!("{:02x}", byte));
    }
    hex_string
}

#[derive(Debug, Clone)]
struct Input {
    prevout: Vec<u8>,
    script_sig: Vec<u8>,
    witness: Vec<Vec<u8>>,
    prevout_data: (Vec<u8>, u32),
}

#[derive(Debug, Clone)]
struct ScanTxHelper {
    ins: Vec<Input>,
    outs: Vec<Vec<u8>>,
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prevout: {}, ", vec_to_hex_string(&self.prevout))?;
        write!(f, "script_sig: {}, ", vec_to_hex_string(&self.script_sig))?;
        for witness_elem in self.witness.iter() {
            write!(f, "witness: {}, ", vec_to_hex_string(&witness_elem))?;
        }
        write!(
            f,
            "prevout txid: {}, ",
            bitcoin::Txid::from_slice(&self.prevout_data.0).unwrap()
        )?;
        write!(f, "prevout n: {}, ", self.prevout_data.1)
    }
}

impl fmt::Display for ScanTxHelper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for input in self.ins.iter() {
            write!(f, "input: {}\n", input)?;
        }
        for output in self.outs.iter() {
            write!(f, "output: {}\n", vec_to_hex_string(&output))?;
        }
        write!(f, "")
    }
}

// silent payment txid:
// 4282b1727f0ebb0c035e8306c2c09764b5b637d63ae5249f7d0d1968a1554231
// silent payment tx:
// 02000000000102bbbd77f0d8c5cbc2ccc39f0501828ad4ac3a6a933393876cae5a7e49bd5341230100000000fdffffff94e299c837e0e00644b9123d80c052159443907f663e746be7fe1e6c32c3ee9b0100000000fdffffff0218e0f50500000000225120d7bf24e13daf4d6ce0ac7a34ecefb4122f070a1561e8659d4071c52edb7c1cb300e1f505000000002251207ef15780916ae0f29a0bd34e48e1a0e817e7731b82f3009cfa89c87602cf1b2b02473044022014680d9a963868b03d25f84bd81af87e127f9d7990166dad5e1dd71be8797e3402205f79713b4faaff7184fb25d0976a37970f8d6b23f95d4041180a35aa291fc8dc012102a9dfaeeebad1f7ebca371a6f02e63a8b0de287c1b0608edc259c60583a03496e0247304402201f09ecdb89f311c3ad8b6d89a040a5796f83c9db2597962969392a3d9a5be46d022052243418a89831ca0e5ddd7ae575d787178126d8495f890414ab8b4d2a1b19d80121035368c752d3ee31d9570180a1ba285659af106f9430811ec58e3b86cf26c208f100000000
// silent payment to address:
// sprt1qqw7zfpjcuwvq4zd3d4aealxq3d669s3kcde4wgr3zl5ugxs40twv2qccgvszutt7p796yg4h926kdnty66wxrfew26gu2gk5h5hcg4s2jqyascfz
// spend key:
// cRFcZbp7cAeZGsnYKdgSZwH6drJ3XLnPSGcjLNCpRy28tpGtZR11
// scan key:
// cTiSJ8p2zpGSkWGkvYFWfKurgWvSi9hdvzw9GEws18kS2VRPNS24

fn parse_keys() -> (Receiver, SecretKey) {
    let original = "sprt1qqw7zfpjcuwvq4zd3d4aealxq3d669s3kcde4wgr3zl5ugxs40twv2qccgvszutt7p796yg4h926kdnty66wxrfew26gu2gk5h5hcg4s2jqyascfz";

    let spend_key =
        PrivateKey::from_wif("cRFcZbp7cAeZGsnYKdgSZwH6drJ3XLnPSGcjLNCpRy28tpGtZR11").unwrap();
    let scan_key =
        PrivateKey::from_wif("cTiSJ8p2zpGSkWGkvYFWfKurgWvSi9hdvzw9GEws18kS2VRPNS24").unwrap();

    let secp = Secp256k1::new();
    let public_spend_key: secp256k1::PublicKey = spend_key.public_key(&secp).inner;
    let public_scan_key: secp256k1::PublicKey = scan_key.public_key(&secp).inner;

    let label = Label::new(spend_key.inner, 0);
    let receiver = Receiver::new(0, public_scan_key, public_spend_key, label, false).unwrap();
    println!("Receiver address: {}", receiver.get_receiving_address());
    println!("Actual adress:    {}", original);
    (receiver, scan_key.inner)
}

fn scan_tx(
    receiver: &Receiver,
    secret_scan_key: &SecretKey,
    scan_tx_helper: &ScanTxHelper,
) -> bool {
    let mut input_pub_keys = Vec::new();

    for input in scan_tx_helper.ins.iter() {
        if let Ok(Some(pubkey)) =
            get_pubkey_from_input(&input.script_sig, &input.witness, &input.prevout)
        {
            input_pub_keys.push(pubkey);
        }
    }

    if input_pub_keys.is_empty() {
        return false;
    }

    let mut outpoints_data = Vec::new();
    for input in scan_tx_helper.ins.iter() {
        if let Ok(txid) = bitcoin::Txid::from_slice(&input.prevout_data.0) {
            outpoints_data.push((txid.to_string(), input.prevout_data.1));
        } else {
            return false;
        }
    }

    let pubkeys_ref: Vec<&PublicKey> = input_pub_keys.iter().collect();
    let tweak_data = match calculate_tweak_data(&pubkeys_ref, &outpoints_data) {
        Ok(data) => data,
        Err(_) => return false,
    };

    let ecdh_shared_secret = match calculate_shared_secret(tweak_data, *secret_scan_key) {
        Ok(secret) => secret,
        Err(_) => return false,
    };

    let mut pubkeys_to_check = Vec::new();

    for script_pubkey in scan_tx_helper.outs.iter() {
        if script_pubkey.len() < 2 {
            continue;
        }

        if let Ok(pubkey) = XOnlyPublicKey::from_slice(&script_pubkey[2..]) {
            pubkeys_to_check.push(pubkey);
        }
    }

    if pubkeys_to_check.is_empty() {
        return false;
    }

    match receiver.scan_transaction(&ecdh_shared_secret, pubkeys_to_check) {
        Ok(res) => {
            info!("res {:?}", res);
            true
        }
        Err(_) => false,
    }
}

fn process_block(
    block_index: &BlockReaderIndex,
    receiver: &Receiver,
    secret_scan_key: &SecretKey,
) -> bool {
    let height = block_index.height();

    let block_ref = match block_index.block() {
        Ok(block) => block,
        Err(_) => return false,
    };

    let block_undo = match block_index.block_undo() {
        Ok(undo) => undo,
        Err(_) => return false,
    };

    let tx_count = block_ref.transaction_count();

    for tx_index in 1..tx_count {
        let transaction = match block_ref.transaction(tx_index) {
            Some(tx) => tx,
            None => {
                error!(
                    "Failed to get transaction {} for block {}",
                    tx_index, height
                );
                return false;
            }
        };

        if transaction.is_coinbase() {
            continue;
        }

        let input_count = transaction.input_count();
        let undo_tx_index = (tx_index - 1) as u64;

        let mut scan_tx_helper = ScanTxHelper {
            ins: Vec::new(),
            outs: Vec::new(),
        };

        for output_index in 0..transaction.output_count() {
            if let Some(output) = transaction.output(output_index) {
                let script_pubkey = output.script_pubkey();
                scan_tx_helper.outs.push(script_pubkey.as_bytes().to_vec());
            } else {
                error!(
                    "Failed to get output {} from transaction {} at block {}",
                    output_index, tx_index, height
                );
                return false;
            }
        }

        for input_index in 0..input_count {
            let input = match transaction.input(input_index) {
                Some(inp) => inp,
                None => {
                    error!(
                        "Failed to get input {} from transaction {} at block {}",
                        input_index, tx_index, height
                    );
                    return false;
                }
            };

            let prevout = match block_undo.prevout_by_index(undo_tx_index, input_index as u64) {
                Some(prevout) => prevout,
                None => {
                    error!(
                        "Failed to get prevout for input {} from transaction {} at height {}",
                        input_index, tx_index, height
                    );
                    return false;
                }
            };

            let script_sig_bytes = input.script_sig().as_bytes().to_vec();

            let witness = input.witness();
            let mut witness_stack = Vec::new();
            if !witness.is_null() {
                for stack_index in 0..witness.stack_size() {
                    if let Some(stack_item) = witness.stack_item(stack_index) {
                        witness_stack.push(stack_item);
                    }
                }
            }

            let outpoint = input.out_point();
            let tx_id = outpoint.tx_id();
            let vout = outpoint.index();

            let prevout_script_pubkey = prevout.script_pubkey();

            scan_tx_helper.ins.push(Input {
                prevout: prevout_script_pubkey.as_bytes().to_vec(),
                script_sig: script_sig_bytes,
                witness: witness_stack,
                prevout_data: (tx_id.hash.to_vec(), vout),
            });
        }

        let scan_success = scan_tx(receiver, secret_scan_key, &scan_tx_helper);
        if scan_success {
            info!(
                "Processed transaction {} at height {} with {} inputs and {} outputs",
                tx_index,
                height,
                scan_tx_helper.ins.len(),
                scan_tx_helper.outs.len()
            );
        }
    }

    true
}

fn main() {
    setup_logger();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_data_dir>", args[0]);
        process::exit(1);
    }

    let data_dir = args[1].clone();

    let reader = BlockReader::new(&data_dir, ChainType::SIGNET).unwrap();

    if let Some(best_block) = reader.best_validated_block_index() {
        let (receiver, secret_scan_key) = parse_keys();

        let start_height = best_block.height().saturating_sub(10);
        let current_block_index = match reader.block_index_at(start_height) {
            Some(block_index) => block_index,
            None => {
                error!("Could not find block index at {}", start_height);
                return;
            }
        };

        process_block(&current_block_index, &receiver, &secret_scan_key);

        for block_index in current_block_index.iter_forwards().take(100) {
            process_block(&block_index, &receiver, &secret_scan_key);
        }
    }
}
