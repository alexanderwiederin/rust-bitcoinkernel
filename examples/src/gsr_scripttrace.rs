use bitcoin::{
    absolute::LockTime,
    consensus::serialize,
    opcodes::all as opcodes,
    taproot::{ControlBlock, LeafVersion, TaprootBuilder},
    transaction::Version,
    Amount, ScriptBuf, XOnlyPublicKey,
};
use bitcoinkernel::{
    core::{TransactionExt, TxOutExt},
    verify, KernelError, PrecomputedTransactionData, ScriptEvalStackRef, ScriptTraceFrameKind,
    ScriptTraceFrameRef, ScriptTracer, ScriptVerificationFlags, Transaction, VERIFY_ALL,
    VERIFY_P2SH, VERIFY_TAPROOT, VERIFY_WITNESS,
};
use secp256k1::Secp256k1;

const TAPROOT_LEAF_TAPSCRIPT_V2: u8 = 0xc2;

const NUMS: [u8; 32] = [
    0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
    0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
];

const AMOUNT: Amount = Amount::from_sat(50_000);

const VERIFY_NO_RESTORATION: ScriptVerificationFlags =
    VERIFY_P2SH | VERIFY_WITNESS | VERIFY_TAPROOT;

fn main() {
    let _tracer = ScriptTracer::new(print_frame).expect("failed to register the script tracer");
    let script = ScriptBuf::builder()
        .push_opcode(opcodes::OP_CAT)
        .into_script();
    let stack = vec![vec![0x01u8], vec![0x02u8]];

    println!("### tapscript v2, restoration enabled ###");
    {
        let result = run(&script, &stack, VERIFY_ALL);
        println!("verify -> {result:?}\n");
    }

    // Same spend without the restoration flag: the 0xc2 leaf is an unknown
    // leaf version, so it succeeds without executing anything. Expect no
    // frames at all.
    println!("### same spend, restoration disabled (expect no frames) ###");
    {
        let result = run(&script, &stack, VERIFY_NO_RESTORATION);
        println!("verify -> {result:?}\n");
    }

    // --- case 2: a script that fails ---------------------------------------
    // OP_CAT with only one item on the stack: INVALID_STACK_OPERATION.
    println!("### tapscript v2, failing script ###");
    {
        let result = run(&script, &[vec![0x01u8]], VERIFY_ALL);
        println!("verify -> {result:?}\n");
    }

    // --- case 3: leaves the stack empty -----------------------------------
    // NOTE: this fails CLEANSTACK, but that check runs *after* the trace
    // scope is destroyed, so the END frame will report script_error=0.
    let drop_script = ScriptBuf::builder()
        .push_opcode(opcodes::OP_CAT)
        .push_opcode(opcodes::OP_DROP)
        .into_script();
    println!("### tapscript v2, cleanstack failure (END frame lies) ###");
    {
        let result = run(&drop_script, &stack, VERIFY_ALL);
        println!("verify -> {result:?}\n");
    }
}

fn print_frame(frame: ScriptTraceFrameRef<'_>) {
    match frame.kind() {
        ScriptTraceFrameKind::Begin => {
            let script_len = frame.script().map(|script| script.len()).unwrap_or(0);
            println!(
                "== begin: {} byte script, sig_version {:?}, tapleaf {} ==",
                script_len,
                frame.sig_version(),
                frame
                    .tapleaf_hash()
                    .map(hex::encode)
                    .unwrap_or_else(|| "<none>".into())
            );
            dump_stacks(frame);
        }
        ScriptTraceFrameKind::Step => {
            println!(
                "[step {}] opcode=0x{:02x} exec={}",
                frame.opcode_pos(),
                frame.opcode(),
                frame.exec()
            );
            dump_stacks(frame);
        }
        ScriptTraceFrameKind::End => {
            println!("== end: script_error={} ==", frame.script_error());
            dump_stacks(frame);
        }
    }
}

fn dump_stacks(frame: ScriptTraceFrameRef<'_>) {
    let stack = frame.stack();
    if stack.is_empty() {
        println!("    stack: <empty>");
    }
    dump_stack("stack", stack);
    dump_stack("altstack", frame.altstack());
}

fn dump_stack(label: &str, stack: ScriptEvalStackRef<'_>) {
    for (i, item) in stack.iter().enumerate() {
        match item.to_bytes() {
            Ok(bytes) if bytes.is_empty() => println!("    {label}[{i}]: <empty>"),
            Ok(bytes) => println!("    {label}[{i}]: {}", hex::encode(bytes)),
            Err(err) => println!("    {label}[{i}]: <unavailable: {err}>"),
        }
    }
}

fn run(
    script: &ScriptBuf,
    stack: &[Vec<u8>],
    flags: ScriptVerificationFlags,
) -> Result<(), KernelError> {
    let (spk, control_block) = taproot_output(script);

    let credit = bitcoin::Transaction {
        version: Version::ONE,
        lock_time: LockTime::ZERO,
        input: vec![bitcoin::TxIn {
            previous_output: bitcoin::OutPoint::null(),
            script_sig: ScriptBuf::builder().push_int(0).push_int(0).into_script(),
            sequence: bitcoin::Sequence::MAX,
            witness: bitcoin::Witness::new(),
        }],
        output: vec![bitcoin::TxOut {
            value: AMOUNT,
            script_pubkey: spk,
        }],
    };

    let mut witness = bitcoin::Witness::new();
    for item in stack {
        witness.push(item);
    }
    witness.push(script.as_bytes());
    witness.push(control_block.serialize());

    let spend = bitcoin::Transaction {
        version: Version::ONE,
        lock_time: LockTime::ZERO,
        input: vec![bitcoin::TxIn {
            previous_output: bitcoin::OutPoint {
                txid: credit.txid(),
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: bitcoin::Sequence::MAX,
            witness,
        }],
        output: vec![bitcoin::TxOut {
            value: AMOUNT,
            script_pubkey: ScriptBuf::new(),
        }],
    };

    let credit_tx = Transaction::new(serialize(&credit).as_slice()).unwrap();
    let spend_tx = Transaction::new(serialize(&spend).as_slice()).unwrap();

    let spent_output = credit_tx.output(0).unwrap();
    let tx_data = PrecomputedTransactionData::new(&spend_tx, &[spent_output]).unwrap();

    verify(
        &spent_output.script_pubkey(),
        Some(AMOUNT.to_sat() as i64),
        &spend_tx,
        0,
        Some(flags),
        &tx_data,
    )
}

fn taproot_output(script: &ScriptBuf) -> (ScriptBuf, ControlBlock) {
    let secp = Secp256k1::new();
    let internal_key = XOnlyPublicKey::from_slice(&NUMS).expect("valid nums point");
    let leaf_version = LeafVersion::from_consensus(TAPROOT_LEAF_TAPSCRIPT_V2)
        .expect("0xc2 is a valid leaf version");

    let spend_info = TaprootBuilder::new()
        .add_leaf_with_ver(0, script.clone(), leaf_version)
        .expect("single leaf at depth 0")
        .finalize(&secp, internal_key)
        .expect("finalize single-leaf tree");

    let control_block = spend_info
        .control_block(&(script.clone(), leaf_version))
        .expect("leaf is in the tree");

    let spk = ScriptBuf::new_p2tr_tweaked(spend_info.output_key());

    (spk, control_block)
}
