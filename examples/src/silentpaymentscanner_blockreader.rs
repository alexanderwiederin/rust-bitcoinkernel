use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fmt::format;
use std::process;

use bitcoin::hashes::Hash;
use bitcoin::{PrivateKey, XOnlyPublicKey};
use bitcoinkernel::BlockReader;
use bitcoinkernel::BlockReaderIndex;
use bitcoinkernel::{ChainType, KernelError, Log, Logger};
use env_logger::Builder;
use log::LevelFilter;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use silentpayments::receiving::{Label, Receiver};
use silentpayments::utils::receiving::{
    calculate_shared_secret, calculate_tweak_data, get_pubkey_from_input,
};

struct MainLog {}

impl Log for MainLog {
    fn log(&self, message: &str) {
        log::info!(
            target: "libbitcoinkernel", 
            "{}", message.strip_suffix("\r\n").or_else(|| message.strip_suffix('\n')).unwrap_or(message));
    }
}

fn setup_logging() -> Result<Logger<MainLog>, KernelError> {
    let mut builder = Builder::from_default_env();
    builder.filter(None, LevelFilter::Trace).init();
    Logger::new(MainLog {})
}

fn vec_to_hex_string(data: &Vec<u8>) -> String {
    let mut hex_string = String::with_capacity(data.len() * 2);
    for byte in data {
        hex_string.push_str(&format!("{:02x}", byte));
    }
    hex_string
}

#[derive(Debug)]
pub struct ProcessingError {
    pub height: i32,
    pub tx_index: Option<usize>,
    pub tx_id: Option<String>,
    pub input_index: Option<usize>,
    pub error_type: ErrorType,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorType {
    BlockRead,
    UndoDataRead,
    TransactionAccess,
    PrevoutAccess,
    ScanTransaction,
    CountMismatch,
    NoValidPublicKeys,
    TweakDataCalculation,
    SharedSecretCalculation,
    PublicKeyExtraction,
    InvalidTxid,
    InvalidXOnlyPublicKey,
    TransactionScan,
}

#[derive(Debug, Clone)]
pub struct ScanContext {
    pub height: i32,
    pub tx_index: usize,
    pub tx_id: String,
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
    context: &ScanContext,
    errors: &mut Vec<ProcessingError>,
) -> bool {
    let mut input_pub_keys = Vec::new();
    let mut pubkey_extraction_errors = 0;

    for (input_index, input) in scan_tx_helper.ins.iter().enumerate() {
        match get_pubkey_from_input(&input.script_sig, &input.witness, &input.prevout) {
            Ok(Some(pubkey)) => input_pub_keys.push(pubkey),
            Ok(None) => {
                errors.push(ProcessingError {
                    height: context.height,
                    tx_index: Some(context.tx_index),
                    tx_id: Some(context.tx_id.clone()),
                    input_index: Some(input_index),
                    error_type: ErrorType::PublicKeyExtraction,
                    message: format!("No valid public key found in input {}", input_index),
                });
            }
            Err(e) => {
                errors.push(ProcessingError {
                    height: context.height,
                    tx_index: Some(context.tx_index),
                    tx_id: Some(context.tx_id.clone()),
                    input_index: Some(input_index),
                    error_type: ErrorType::PublicKeyExtraction,
                    message: format!(
                        "Failed to extract public key from input {}: {}",
                        input_index, e
                    ),
                });
                pubkey_extraction_errors += 1;
            }
        }
    }

    if input_pub_keys.is_empty() {
        errors.push(ProcessingError {
            height: context.height,
            tx_index: Some(context.tx_index),
            tx_id: Some(context.tx_id.clone()),
            input_index: None,
            error_type: ErrorType::NoValidPublicKeys,
            message: format!(
                "No valid public keys found in transaction (feiled ot extract from {} inputs)",
                pubkey_extraction_errors,
            ),
        });
        return false;
    }

    let mut outpoints_data = Vec::new();
    for (input_index, input) in scan_tx_helper.ins.iter().enumerate() {
        match bitcoin::Txid::from_slice(&input.prevout_data.0) {
            Ok(txid) => {
                outpoints_data.push((txid.to_string(), input.prevout_data.1));
            }
            Err(e) => {
                errors.push(ProcessingError {
                    height: context.height,
                    tx_index: Some(context.tx_index),
                    tx_id: Some(context.tx_id.clone()),
                    input_index: Some(input_index),
                    error_type: ErrorType::InvalidTxid,
                    message: format!(
                        "Invalid transaction ID format for input {}: {}",
                        input_index, e
                    ),
                });
                return false;
            }
        }
    }

    let pubkeys_ref: Vec<&PublicKey> = input_pub_keys.iter().collect();
    let tweak_data = match calculate_tweak_data(&pubkeys_ref, &outpoints_data) {
        Ok(data) => data,
        Err(e) => {
            errors.push(ProcessingError {
                height: context.height,
                tx_index: Some(context.tx_index),
                tx_id: Some(context.tx_id.clone()),
                input_index: None,
                error_type: ErrorType::TweakDataCalculation,
                message: format!("Failed to calculate tweak data: {:?}", e),
            });
            return false;
        }
    };

    let ecdh_shared_secret = match calculate_shared_secret(tweak_data, *secret_scan_key) {
        Ok(secret) => secret,
        Err(e) => {
            errors.push(ProcessingError {
                height: context.height,
                tx_index: Some(context.tx_index),
                tx_id: Some(context.tx_id.clone()),
                input_index: None,
                error_type: ErrorType::SharedSecretCalculation,
                message: format!("Failed to calculate shared secret: {}", e),
            });
            return false;
        }
    };

    let mut pubkeys_to_check = Vec::new();
    let mut invalid_pubkey_count = 0;

    for (output_index, script_pubkey) in scan_tx_helper.outs.iter().enumerate() {
        if script_pubkey.len() < 2 {
            errors.push(ProcessingError {
                height: context.height,
                tx_index: Some(context.tx_index),
                tx_id: Some(context.tx_id.clone()),
                input_index: None,
                error_type: ErrorType::InvalidXOnlyPublicKey,
                message: format!(
                    "Script pubkey too short for output {}: {} bytes",
                    output_index,
                    script_pubkey.len()
                ),
            });
            invalid_pubkey_count += 1;
            continue;
        }

        match XOnlyPublicKey::from_slice(&script_pubkey[2..]) {
            Ok(pubkey) => pubkeys_to_check.push(pubkey),
            Err(e) => {
                errors.push(ProcessingError {
                    height: context.height,
                    tx_index: Some(context.tx_index),
                    tx_id: Some(context.tx_id.clone()),
                    input_index: None,
                    error_type: ErrorType::InvalidXOnlyPublicKey,
                    message: format!(
                        "Invalid XOnlyPublicKey format for output {}, {}",
                        output_index, e
                    ),
                });
                invalid_pubkey_count += 1;
            }
        }
    }

    if invalid_pubkey_count > 0 && pubkeys_to_check.is_empty() {
        errors.push(ProcessingError {
            height: context.height,
            tx_index: Some(context.tx_index),
            tx_id: Some(context.tx_id.clone()),
            input_index: None,
            error_type: ErrorType::InvalidXOnlyPublicKey,
            message: format!(
                "No valid XOnlyPublicKeys found in {} outputs",
                scan_tx_helper.outs.len()
            ),
        });
        return false;
    }

    match receiver.scan_transaction(&ecdh_shared_secret, pubkeys_to_check) {
        Ok(res) => {
            println!("/nres {:?}\n", res);
            true
        }
        Err(e) => {
            errors.push(ProcessingError {
                height: context.height,
                tx_index: Some(context.tx_index),
                tx_id: Some(context.tx_id.clone()),
                input_index: None,
                error_type: ErrorType::TransactionScan,
                message: format!("Transaction scan failed: {}", e),
            });
            false
        }
    }
}

fn process_block_with_error_collection(
    block_index: &BlockReaderIndex,
    receiver: &Receiver,
    secret_scan_key: &SecretKey,
) -> Vec<ProcessingError> {
    let mut errors = Vec::new();
    let height = block_index.height();

    let block_ref = match block_index.block() {
        Ok(block) => block,
        Err(e) => {
            errors.push(ProcessingError {
                height,
                tx_index: None,
                tx_id: None,
                input_index: None,
                error_type: ErrorType::BlockRead,
                message: e.to_string(),
            });
            return errors;
        }
    };

    let block_undo = match block_index.block_undo() {
        Ok(undo) => undo,
        Err(e) => {
            errors.push(ProcessingError {
                height,
                tx_index: None,
                tx_id: None,
                input_index: None,
                error_type: ErrorType::UndoDataRead,
                message: e.to_string(),
            });
            return errors;
        }
    };

    let tx_count = block_ref.transaction_count();
    let undo_tx_count = block_undo.transaction_count() as usize;

    if tx_count > 1 && tx_count - 1 != undo_tx_count {
        errors.push(ProcessingError {
            height,
            tx_index: None,
            tx_id: None,
            input_index: None,
            error_type: ErrorType::CountMismatch,
            message: format!(
                "Transaction cout mismatch: {} vs {}",
                tx_count - 1,
                undo_tx_count
            ),
        });
    }

    for tx_index in 1..tx_count {
        let transaction = match block_ref.transaction(tx_index) {
            Some(tx) => tx,
            None => {
                errors.push(ProcessingError {
                    height,
                    tx_index: Some(tx_index),
                    tx_id: None,
                    input_index: None,
                    error_type: ErrorType::TransactionAccess,
                    message: format!("Failed to get transaction {}", tx_index),
                });
                continue;
            }
        };

        if transaction.is_coinbase() {
            continue;
        }

        let input_count = transaction.input_count();
        let undo_tx_index = (tx_index - 1) as u64;
        let undo_input_count = block_undo.transaction_undo_size(undo_tx_index);
        assert_eq!(
            input_count as u64,
            undo_input_count,
            "Input count mismatch for tx {} at height {}",
            tx_index,
            block_index.height()
        );

        let mut scan_tx_helper = ScanTxHelper {
            ins: Vec::new(),
            outs: Vec::new(),
        };

        for output_index in 0..transaction.output_count() {
            if let Some(output) = transaction.output(output_index) {
                let script_pubkey = output.script_pubkey();
                scan_tx_helper.outs.push(script_pubkey.as_bytes().to_vec());
            }
        }

        for input_index in 0..input_count {
            let input = match transaction.input(input_index) {
                Some(inp) => inp,
                None => continue,
            };

            let prevout = match block_undo.prevout_by_index(undo_tx_index, input_index as u64) {
                Ok(prevout) => prevout,
                Err(e) => {
                    errors.push(ProcessingError {
                        height,
                        tx_index: Some(tx_index),
                        tx_id: Some(transaction.hash().display_order()),
                        input_index: Some(input_index),
                        error_type: ErrorType::PrevoutAccess,
                        message: e.to_string(),
                    });
                    continue;
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
        let scan_context = ScanContext {
            tx_index,
            height,
            tx_id: transaction.hash().display_order(),
        };

        let scan_success = scan_tx(
            receiver,
            secret_scan_key,
            &scan_tx_helper,
            &scan_context,
            &mut errors,
        );
        if scan_success {
            println!(
                "Processed transaction {} at height {} with {} inputs and {} outputs",
                tx_index,
                height,
                scan_tx_helper.ins.len(),
                scan_tx_helper.outs.len()
            );
        }
    }

    errors
}

fn log_all_errors(all_errors: &[ProcessingError]) {
    if all_errors.is_empty() {
        println!("✅ No errors encountered during processing!");
        return;
    }
    println!(
        "\n🚨 Processing completed with {} errors:",
        all_errors.len()
    );
    let mut error_counts: HashMap<ErrorType, usize> = HashMap::new();
    for error in all_errors {
        *error_counts.entry(error.error_type.clone()).or_insert(0) += 1;
    }
    println!("\n📊 Error Summary:");
    for (error_type, count) in &error_counts {
        println!("  {:?}: {} occurrences", error_type, count);
    }
    println!("\n📝 Detailed Error Log:");
    for (i, error) in all_errors.iter().enumerate() {
        let mut location_parts = Vec::new();

        if let Some(tx) = error.tx_index {
            location_parts.push(format!("tx:{}", tx));
        }

        if let Some(tx_id) = &error.tx_id {
            location_parts.push(format!("id:{}", tx_id));
        }

        if let Some(input) = error.input_index {
            location_parts.push(format!("input:{}", input));
        }

        let location = if location_parts.is_empty() {
            String::new()
        } else {
            format!(" [{}]", location_parts.join(" "))
        };

        println!(
            "{}. Height {}{}: {} - {}",
            i + 1,
            error.height,
            location,
            format!("{:?}", error.error_type),
            error.message
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_data_dir>", args[0]);
        process::exit(1);
    }

    let _ = setup_logging().unwrap();
    let data_dir = args[1].clone();

    let reader = BlockReader::new(&data_dir, ChainType::SIGNET).unwrap();
    let mut all_errors = Vec::new();

    if let Some(best_block) = reader.best_validated_block() {
        let (receiver, secret_scan_key) = parse_keys();
        let mut current_block_index = best_block;

        let mut block_errors =
            process_block_with_error_collection(&current_block_index, &receiver, &secret_scan_key);
        all_errors.append(&mut block_errors);

        for _i in 0..1 {
            match current_block_index.previous() {
                Ok(previous_block) => {
                    let mut block_errors = process_block_with_error_collection(
                        &previous_block,
                        &receiver,
                        &secret_scan_key,
                    );
                    all_errors.append(&mut block_errors);
                    current_block_index = previous_block;
                }
                Err(_) => {
                    continue;
                }
            }
        }

        log_all_errors(&all_errors);
    }
}
