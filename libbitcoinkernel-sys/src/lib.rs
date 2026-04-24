#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

// Primitive type aliases - alphabetical order

pub type btck_BlockCheckFlags = u32;
pub type btck_BlockValidationResult = u32;
pub type btck_ChainType = u8;
pub type btck_LogCategory = u8;
pub type btck_LogLevel = u8;
pub type btck_ScriptVerificationFlags = u32;
pub type btck_ScriptVerifyStatus = u8;
pub type btck_SynchronizationState = u8;
pub type btck_ValidationMode = u8;
pub type btck_Warning = u8;

// Opaque types - alphabetical order

#[repr(C)]
pub struct btck_Block {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_BlockHash {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_BlockHeader {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_BlockSpentOutputs {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_BlockTreeEntry {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_BlockValidationState {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_Chain {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_ChainParameters {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_ChainstateManager {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_ChainstateManagerOptions {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_Coin {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_ConsensusParams {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_Context {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_ContextOptions {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_LoggingConnection {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_PrecomputedTransactionData {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_ScriptPubkey {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_Transaction {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_TransactionInput {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_TransactionOutPoint {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_TransactionOutput {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_TransactionSpentOutputs {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct btck_Txid {
    _unused: [u8; 0],
}

// Function-pointer type aliases - alphabetical order

pub type btck_DestroyCallback = Option<unsafe extern "C" fn(user_data: *mut c_void)>;

pub type btck_LogCallback = Option<
    unsafe extern "C" fn(user_data: *mut c_void, message: *const c_char, message_len: usize),
>;

pub type btck_NotifyBlockTip = Option<
    unsafe extern "C" fn(
        user_data: *mut c_void,
        state: btck_SynchronizationState,
        entry: *const btck_BlockTreeEntry,
        verification_progress: f64,
    ),
>;

pub type btck_NotifyFatalError = Option<
    unsafe extern "C" fn(user_data: *mut c_void, message: *const c_char, message_len: usize),
>;

pub type btck_NotifyFlushError = Option<
    unsafe extern "C" fn(user_data: *mut c_void, message: *const c_char, message_len: usize),
>;

pub type btck_NotifyHeaderTip = Option<
    unsafe extern "C" fn(
        user_data: *mut c_void,
        state: btck_SynchronizationState,
        height: i64,
        timestamp: i64,
        presync: c_int,
    ),
>;

pub type btck_NotifyProgress = Option<
    unsafe extern "C" fn(
        user_data: *mut c_void,
        title: *const c_char,
        title_len: usize,
        progress_percent: c_int,
        resume_possible: c_int,
    ),
>;

pub type btck_NotifyWarningSet = Option<
    unsafe extern "C" fn(
        user_data: *mut c_void,
        warning: btck_Warning,
        message: *const c_char,
        message_len: usize,
    ),
>;

pub type btck_NotifyWarningUnset =
    Option<unsafe extern "C" fn(user_data: *mut c_void, warning: btck_Warning)>;

pub type btck_ValidationInterfaceBlockChecked = Option<
    unsafe extern "C" fn(
        user_data: *mut c_void,
        block: *mut btck_Block,
        state: *const btck_BlockValidationState,
    ),
>;

pub type btck_ValidationInterfaceBlockConnected = Option<
    unsafe extern "C" fn(
        user_data: *mut c_void,
        block: *mut btck_Block,
        entry: *const btck_BlockTreeEntry,
    ),
>;

pub type btck_ValidationInterfaceBlockDisconnected = Option<
    unsafe extern "C" fn(
        user_data: *mut c_void,
        block: *mut btck_Block,
        entry: *const btck_BlockTreeEntry,
    ),
>;

pub type btck_ValidationInterfacePoWValidBlock = Option<
    unsafe extern "C" fn(
        user_data: *mut c_void,
        block: *mut btck_Block,
        entry: *const btck_BlockTreeEntry,
    ),
>;

pub type btck_WriteBytes =
    Option<unsafe extern "C" fn(bytes: *const c_void, size: usize, userdata: *mut c_void) -> c_int>;

// These structs are passed by value across the FFI boundary - alphabetical order
// Field order must match C exactly - sizes verified by const assertions below

#[repr(C)]
pub struct btck_LoggingOptions {
    pub log_timestamps: c_int,
    pub log_time_micros: c_int,
    pub log_threadnames: c_int,
    pub log_sourcelocations: c_int,
    pub always_print_category_levels: c_int,
}

#[repr(C)]
pub struct btck_NotificationInterfaceCallbacks {
    pub user_data: *mut c_void,
    pub user_data_destroy: btck_DestroyCallback,
    pub block_tip: btck_NotifyBlockTip,
    pub header_tip: btck_NotifyHeaderTip,
    pub progress: btck_NotifyProgress,
    pub warning_set: btck_NotifyWarningSet,
    pub warning_unset: btck_NotifyWarningUnset,
    pub flush_error: btck_NotifyFlushError,
    pub fatal_error: btck_NotifyFatalError,
}

#[repr(C)]
pub struct btck_ValidationInterfaceCallbacks {
    pub user_data: *mut c_void,
    pub user_data_destroy: btck_DestroyCallback,
    pub block_checked: btck_ValidationInterfaceBlockChecked,
    pub pow_valid_block: btck_ValidationInterfacePoWValidBlock,
    pub block_connected: btck_ValidationInterfaceBlockConnected,
    pub block_disconnected: btck_ValidationInterfaceBlockDisconnected,
}

// Compile-time layout guards - catch any ABI drift
const _: () = {
    assert!(core::mem::size_of::<btck_LoggingOptions>() == 20);
    assert!(core::mem::align_of::<btck_LoggingOptions>() == 4);
    assert!(core::mem::size_of::<btck_NotificationInterfaceCallbacks>() == 72);
    assert!(core::mem::align_of::<btck_NotificationInterfaceCallbacks>() == 8);
    assert!(core::mem::size_of::<btck_ValidationInterfaceCallbacks>() == 48);
    assert!(core::mem::align_of::<btck_ValidationInterfaceCallbacks>() == 8);
};

// extern "C" declarations - grouped by type

extern "C" {

    // --- Transaction --------------------------------------------------------

    pub fn btck_transaction_create(
        raw_transaction: *const c_void,
        raw_transaction_len: usize,
    ) -> *mut btck_Transaction;

    pub fn btck_transaction_copy(transaction: *const btck_Transaction) -> *mut btck_Transaction;

    pub fn btck_transaction_to_bytes(
        transaction: *const btck_Transaction,
        writer: btck_WriteBytes,
        user_data: *mut c_void,
    ) -> c_int;

    pub fn btck_transaction_count_outputs(transaction: *const btck_Transaction) -> usize;

    pub fn btck_transaction_get_output_at(
        transaction: *const btck_Transaction,
        output_index: usize,
    ) -> *const btck_TransactionOutput;

    pub fn btck_transaction_get_input_at(
        transaction: *const btck_Transaction,
        input_index: usize,
    ) -> *const btck_TransactionInput;

    pub fn btck_transaction_count_inputs(transaction: *const btck_Transaction) -> usize;

    pub fn btck_transaction_get_locktime(transaction: *const btck_Transaction) -> u32;

    pub fn btck_transaction_get_txid(transaction: *const btck_Transaction) -> *const btck_Txid;

    pub fn btck_transaction_destroy(transaction: *mut btck_Transaction);

    // --- PrecomputedTransactionData -----------------------------------------

    pub fn btck_precomputed_transaction_data_create(
        tx_to: *const btck_Transaction,
        spent_outputs: *mut *const btck_TransactionOutput,
        spent_outputs_len: usize,
    ) -> *mut btck_PrecomputedTransactionData;

    pub fn btck_precomputed_transaction_data_copy(
        precomputed_txdata: *const btck_PrecomputedTransactionData,
    ) -> *mut btck_PrecomputedTransactionData;

    pub fn btck_precomputed_transaction_data_destroy(
        precomputed_txdata: *mut btck_PrecomputedTransactionData,
    );

    // --- ScriptPubkey -------------------------------------------------------

    pub fn btck_script_pubkey_create(
        script_pubkey: *const c_void,
        script_pubkey_len: usize,
    ) -> *mut btck_ScriptPubkey;

    pub fn btck_script_pubkey_copy(
        script_pubkey: *const btck_ScriptPubkey,
    ) -> *mut btck_ScriptPubkey;

    pub fn btck_script_pubkey_verify(
        script_pubkey: *const btck_ScriptPubkey,
        amount: i64,
        tx_to: *const btck_Transaction,
        precomputed_txdata: *const btck_PrecomputedTransactionData,
        input_index: c_uint,
        flags: btck_ScriptVerificationFlags,
        status: *mut btck_ScriptVerifyStatus,
    ) -> c_int;

    pub fn btck_script_pubkey_to_bytes(
        script_pubkey: *const btck_ScriptPubkey,
        writer: btck_WriteBytes,
        user_data: *mut c_void,
    ) -> c_int;

    pub fn btck_script_pubkey_destroy(script_pubkey: *mut btck_ScriptPubkey);

    // --- TransactionOutput --------------------------------------------------

    pub fn btck_transaction_output_create(
        script_pubkey: *const btck_ScriptPubkey,
        amount: i64,
    ) -> *mut btck_TransactionOutput;

    pub fn btck_transaction_output_get_script_pubkey(
        transaction_output: *const btck_TransactionOutput,
    ) -> *const btck_ScriptPubkey;

    pub fn btck_transaction_output_get_amount(
        transaction_output: *const btck_TransactionOutput,
    ) -> i64;

    pub fn btck_transaction_output_copy(
        transaction_output: *const btck_TransactionOutput,
    ) -> *mut btck_TransactionOutput;

    pub fn btck_transaction_output_destroy(transaction_output: *mut btck_TransactionOutput);

    // --- Logging ------------------------------------------------------------

    pub fn btck_logging_disable();

    pub fn btck_logging_set_options(options: btck_LoggingOptions);

    pub fn btck_logging_set_level_category(category: btck_LogCategory, level: btck_LogLevel);

    pub fn btck_logging_enable_category(category: btck_LogCategory);

    pub fn btck_logging_disable_category(category: btck_LogCategory);

    pub fn btck_logging_connection_create(
        log_callback: btck_LogCallback,
        user_data: *mut c_void,
        user_data_destroy_callback: btck_DestroyCallback,
    ) -> *mut btck_LoggingConnection;

    pub fn btck_logging_connection_destroy(logging_connection: *mut btck_LoggingConnection);

    // --- ChainParameters ----------------------------------------------------

    pub fn btck_chain_parameters_create(chain_type: btck_ChainType) -> *mut btck_ChainParameters;

    pub fn btck_chain_parameters_copy(
        chain_parameters: *const btck_ChainParameters,
    ) -> *mut btck_ChainParameters;

    pub fn btck_chain_parameters_get_consensus_params(
        chain_parameters: *const btck_ChainParameters,
    ) -> *const btck_ConsensusParams;

    pub fn btck_chain_parameters_destroy(chain_parameters: *mut btck_ChainParameters);

    // --- ContextOptions -----------------------------------------------------

    pub fn btck_context_options_create() -> *mut btck_ContextOptions;

    pub fn btck_context_options_set_chainparams(
        context_options: *mut btck_ContextOptions,
        chain_parameters: *const btck_ChainParameters,
    );

    pub fn btck_context_options_set_notifications(
        context_options: *mut btck_ContextOptions,
        notifications: btck_NotificationInterfaceCallbacks,
    );

    pub fn btck_context_options_set_validation_interface(
        context_options: *mut btck_ContextOptions,
        validation_interface_callbacks: btck_ValidationInterfaceCallbacks,
    );

    pub fn btck_context_options_destroy(context_options: *mut btck_ContextOptions);

    // --- Context ------------------------------------------------------------

    pub fn btck_context_create(context_options: *const btck_ContextOptions) -> *mut btck_Context;

    pub fn btck_context_copy(context: *const btck_Context) -> *mut btck_Context;

    pub fn btck_context_interrupt(context: *mut btck_Context) -> c_int;

    pub fn btck_context_destroy(context: *mut btck_Context);

    // --- BlockTreeEntry -----------------------------------------------------

    pub fn btck_block_tree_entry_get_previous(
        block_tree_entry: *const btck_BlockTreeEntry,
    ) -> *const btck_BlockTreeEntry;

    pub fn btck_block_tree_entry_get_ancestor(
        block_tree_entry: *const btck_BlockTreeEntry,
        height: i32,
    ) -> *const btck_BlockTreeEntry;

    pub fn btck_block_tree_entry_get_block_header(
        block_tree_entry: *const btck_BlockTreeEntry,
    ) -> *mut btck_BlockHeader;

    pub fn btck_block_tree_entry_get_height(block_tree_entry: *const btck_BlockTreeEntry) -> i32;

    pub fn btck_block_tree_entry_get_block_hash(
        block_tree_entry: *const btck_BlockTreeEntry,
    ) -> *const btck_BlockHash;

    pub fn btck_block_tree_entry_equals(
        entry1: *const btck_BlockTreeEntry,
        entry2: *const btck_BlockTreeEntry,
    ) -> c_int;

    // --- ChainstateManagerOptions -------------------------------------------

    pub fn btck_chainstate_manager_options_create(
        context: *const btck_Context,
        data_directory: *const c_char,
        data_directory_len: usize,
        blocks_directory: *const c_char,
        blocks_directory_len: usize,
    ) -> *mut btck_ChainstateManagerOptions;

    pub fn btck_chainstate_manager_options_set_worker_threads_num(
        chainstate_manager_options: *mut btck_ChainstateManagerOptions,
        worker_threads: c_int,
    );

    pub fn btck_chainstate_manager_options_set_wipe_dbs(
        chainstate_manager_options: *mut btck_ChainstateManagerOptions,
        wipe_block_tree_db: c_int,
        wipe_chainstate_db: c_int,
    ) -> c_int;

    pub fn btck_chainstate_manager_options_update_block_tree_db_in_memory(
        chainstate_manager_options: *mut btck_ChainstateManagerOptions,
        block_tree_db_in_memory: c_int,
    );

    pub fn btck_chainstate_manager_options_update_chainstate_db_in_memory(
        chainstate_manager_options: *mut btck_ChainstateManagerOptions,
        chainstate_db_in_memory: c_int,
    );

    pub fn btck_chainstate_manager_options_destroy(
        chainstate_manager_options: *mut btck_ChainstateManagerOptions,
    );

    // --- ChainstateManager --------------------------------------------------

    pub fn btck_chainstate_manager_create(
        chainstate_manager_options: *const btck_ChainstateManagerOptions,
    ) -> *mut btck_ChainstateManager;

    pub fn btck_chainstate_manager_get_best_entry(
        chainstate_manager: *const btck_ChainstateManager,
    ) -> *const btck_BlockTreeEntry;

    pub fn btck_chainstate_manager_process_block_header(
        chainstate_manager: *mut btck_ChainstateManager,
        header: *const btck_BlockHeader,
        block_validation_state: *mut btck_BlockValidationState,
    ) -> c_int;

    pub fn btck_chainstate_manager_import_blocks(
        chainstate_manager: *mut btck_ChainstateManager,
        block_file_paths_data: *mut *const c_char,
        block_file_paths_lens: *mut usize,
        block_file_paths_data_len: usize,
    ) -> c_int;

    pub fn btck_chainstate_manager_process_block(
        chainstate_manager: *mut btck_ChainstateManager,
        block: *const btck_Block,
        new_block: *mut c_int,
    ) -> c_int;

    pub fn btck_chainstate_manager_get_active_chain(
        chainstate_manager: *const btck_ChainstateManager,
    ) -> *const btck_Chain;

    pub fn btck_chainstate_manager_get_block_tree_entry_by_hash(
        chainstate_manager: *const btck_ChainstateManager,
        block_hash: *const btck_BlockHash,
    ) -> *const btck_BlockTreeEntry;

    pub fn btck_chainstate_manager_destroy(chainstate_manager: *mut btck_ChainstateManager);

    // --- Block --------------------------------------------------------------

    pub fn btck_block_read(
        chainstate_manager: *const btck_ChainstateManager,
        block_tree_entry: *const btck_BlockTreeEntry,
    ) -> *mut btck_Block;

    pub fn btck_block_create(raw_block: *const c_void, raw_block_len: usize) -> *mut btck_Block;

    pub fn btck_block_copy(block: *const btck_Block) -> *mut btck_Block;

    pub fn btck_block_check(
        block: *const btck_Block,
        consensus_params: *const btck_ConsensusParams,
        flags: btck_BlockCheckFlags,
        validation_state: *mut btck_BlockValidationState,
    ) -> c_int;

    pub fn btck_block_count_transactions(block: *const btck_Block) -> usize;

    pub fn btck_block_get_transaction_at(
        block: *const btck_Block,
        transaction_index: usize,
    ) -> *const btck_Transaction;

    pub fn btck_block_get_header(block: *const btck_Block) -> *mut btck_BlockHeader;

    pub fn btck_block_get_hash(block: *const btck_Block) -> *mut btck_BlockHash;

    pub fn btck_block_to_bytes(
        block: *const btck_Block,
        writer: btck_WriteBytes,
        user_data: *mut c_void,
    ) -> c_int;

    pub fn btck_block_destroy(block: *mut btck_Block);

    // --- BlockValidationState -----------------------------------------------

    pub fn btck_block_validation_state_create() -> *mut btck_BlockValidationState;

    pub fn btck_block_validation_state_get_validation_mode(
        block_validation_state: *const btck_BlockValidationState,
    ) -> btck_ValidationMode;

    pub fn btck_block_validation_state_get_block_validation_result(
        block_validation_state: *const btck_BlockValidationState,
    ) -> btck_BlockValidationResult;

    pub fn btck_block_validation_state_copy(
        block_validation_state: *const btck_BlockValidationState,
    ) -> *mut btck_BlockValidationState;

    pub fn btck_block_validation_state_destroy(
        block_validation_state: *mut btck_BlockValidationState,
    );

    // --- Chain --------------------------------------------------------------

    pub fn btck_chain_get_height(chain: *const btck_Chain) -> i32;

    pub fn btck_chain_get_by_height(
        chain: *const btck_Chain,
        block_height: i32,
    ) -> *const btck_BlockTreeEntry;

    pub fn btck_chain_contains(
        chain: *const btck_Chain,
        block_tree_entry: *const btck_BlockTreeEntry,
    ) -> c_int;

    // --- BlockSpentOutputs --------------------------------------------------

    pub fn btck_block_spent_outputs_read(
        chainstate_manager: *const btck_ChainstateManager,
        block_tree_entry: *const btck_BlockTreeEntry,
    ) -> *mut btck_BlockSpentOutputs;

    pub fn btck_block_spent_outputs_copy(
        block_spent_outputs: *const btck_BlockSpentOutputs,
    ) -> *mut btck_BlockSpentOutputs;

    pub fn btck_block_spent_outputs_count(
        block_spent_outputs: *const btck_BlockSpentOutputs,
    ) -> usize;

    pub fn btck_block_spent_outputs_get_transaction_spent_outputs_at(
        block_spent_outputs: *const btck_BlockSpentOutputs,
        transaction_spent_outputs_index: usize,
    ) -> *const btck_TransactionSpentOutputs;

    pub fn btck_block_spent_outputs_destroy(block_spent_outputs: *mut btck_BlockSpentOutputs);

    // --- TransactionSpentOutputs --------------------------------------------

    pub fn btck_transaction_spent_outputs_copy(
        transaction_spent_outputs: *const btck_TransactionSpentOutputs,
    ) -> *mut btck_TransactionSpentOutputs;

    pub fn btck_transaction_spent_outputs_count(
        transaction_spent_outputs: *const btck_TransactionSpentOutputs,
    ) -> usize;

    pub fn btck_transaction_spent_outputs_get_coin_at(
        transaction_spent_outputs: *const btck_TransactionSpentOutputs,
        coin_index: usize,
    ) -> *const btck_Coin;

    pub fn btck_transaction_spent_outputs_destroy(
        transaction_spent_outputs: *mut btck_TransactionSpentOutputs,
    );

    // --- TransactionInput ---------------------------------------------------

    pub fn btck_transaction_input_copy(
        transaction_input: *const btck_TransactionInput,
    ) -> *mut btck_TransactionInput;

    pub fn btck_transaction_input_get_out_point(
        transaction_input: *const btck_TransactionInput,
    ) -> *const btck_TransactionOutPoint;

    pub fn btck_transaction_input_get_sequence(
        transaction_input: *const btck_TransactionInput,
    ) -> u32;

    pub fn btck_transaction_input_destroy(transaction_input: *mut btck_TransactionInput);

    // --- TransactionOutPoint ------------------------------------------------

    pub fn btck_transaction_out_point_copy(
        transaction_out_point: *const btck_TransactionOutPoint,
    ) -> *mut btck_TransactionOutPoint;

    pub fn btck_transaction_out_point_get_index(
        transaction_out_point: *const btck_TransactionOutPoint,
    ) -> u32;

    pub fn btck_transaction_out_point_get_txid(
        transaction_out_point: *const btck_TransactionOutPoint,
    ) -> *const btck_Txid;

    pub fn btck_transaction_out_point_destroy(transaction_out_point: *mut btck_TransactionOutPoint);

    // --- Txid ---------------------------------------------------------------

    pub fn btck_txid_copy(txid: *const btck_Txid) -> *mut btck_Txid;

    pub fn btck_txid_equals(txid1: *const btck_Txid, txid2: *const btck_Txid) -> c_int;

    pub fn btck_txid_to_bytes(txid: *const btck_Txid, output: *mut c_uchar);

    pub fn btck_txid_destroy(txid: *mut btck_Txid);

    // --- Coin ---------------------------------------------------------------

    pub fn btck_coin_copy(coin: *const btck_Coin) -> *mut btck_Coin;

    pub fn btck_coin_confirmation_height(coin: *const btck_Coin) -> u32;

    pub fn btck_coin_is_coinbase(coin: *const btck_Coin) -> c_int;

    pub fn btck_coin_get_output(coin: *const btck_Coin) -> *const btck_TransactionOutput;

    pub fn btck_coin_destroy(coin: *mut btck_Coin);

    // --- BlockHash ----------------------------------------------------------

    pub fn btck_block_hash_create(block_hash: *const c_uchar) -> *mut btck_BlockHash;

    pub fn btck_block_hash_equals(
        hash1: *const btck_BlockHash,
        hash2: *const btck_BlockHash,
    ) -> c_int;

    pub fn btck_block_hash_copy(block_hash: *const btck_BlockHash) -> *mut btck_BlockHash;

    pub fn btck_block_hash_to_bytes(block_hash: *const btck_BlockHash, output: *mut c_uchar);

    pub fn btck_block_hash_destroy(block_hash: *mut btck_BlockHash);

    // --- BlockHeader --------------------------------------------------------

    pub fn btck_block_header_create(
        raw_block_header: *const c_void,
        raw_block_header_len: usize,
    ) -> *mut btck_BlockHeader;

    pub fn btck_block_header_copy(header: *const btck_BlockHeader) -> *mut btck_BlockHeader;

    pub fn btck_block_header_get_hash(header: *const btck_BlockHeader) -> *mut btck_BlockHash;

    pub fn btck_block_header_get_prev_hash(
        header: *const btck_BlockHeader,
    ) -> *const btck_BlockHash;

    pub fn btck_block_header_get_timestamp(header: *const btck_BlockHeader) -> u32;

    pub fn btck_block_header_get_bits(header: *const btck_BlockHeader) -> u32;

    pub fn btck_block_header_get_version(header: *const btck_BlockHeader) -> i32;

    pub fn btck_block_header_get_nonce(header: *const btck_BlockHeader) -> u32;

    pub fn btck_block_header_to_bytes(
        header: *const btck_BlockHeader,
        output: *mut c_uchar,
    ) -> c_int;

    pub fn btck_block_header_destroy(header: *mut btck_BlockHeader);

} // extern "C"
