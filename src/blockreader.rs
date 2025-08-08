use libbitcoinkernel_sys::*;
use std::{
    ffi::CString,
    fmt::{self},
    marker::PhantomData,
    sync::Arc,
};
use thiserror::Error;

use crate::{ChainParams, ChainType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IBDStatus {
    NoData,
    InIBD,
    Synced,
}

impl From<kernel_blockreader_IBDStatus> for IBDStatus {
    fn from(status: kernel_blockreader_IBDStatus) -> Self {
        match status {
            kernel_blockreader_IBDStatus_kernel_blockreader_IBD_STATUS_NO_DATA => IBDStatus::NoData,
            kernel_blockreader_IBDStatus_kernel_blockreader_IBD_STATUS_IN_IBD => IBDStatus::InIBD,
            kernel_blockreader_IBDStatus_kernel_blockreader_IBD_STATUS_SYNCED => IBDStatus::Synced,
            _ => IBDStatus::NoData,
        }
    }
}

#[derive(Debug, Error)]
pub enum BlockReaderError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Failed to read block at height {0}")]
    ReadError(i32),

    #[error("Chain parameters error: {0}")]
    ChainParamsError(String),

    #[error("Index out of bounds")]
    OutOfBounds,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Hash {
    pub hash: [u8; 32],
}

impl Hash {
    pub fn display_order(&self) -> String {
        self.hash
            .iter()
            .rev()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>()
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_string = self
            .hash
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();
        write!(f, "{}", hex_string)
    }
}

#[derive(Debug, Clone)]
pub struct ScriptPubkeyRef {
    inner: *const kernel_ScriptPubkey,
    marker: PhantomData<BlockRef>,
}

impl ScriptPubkeyRef {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let data_ptr = kernel_script_pubkey_get_data(self.inner);
            let size = kernel_script_pubkey_get_size(self.inner);
            if data_ptr == std::ptr::null() || size == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(data_ptr, size)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptSigRef {
    inner: *const kernel_TransactionScriptSig,
    marker: PhantomData<BlockRef>,
}

impl ScriptSigRef {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let data_ptr = kernel_script_sig_get_data(self.inner);
            let size = kernel_script_sig_get_size(self.inner);
            if data_ptr == std::ptr::null() || size == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(data_ptr, size)
            }
        }
    }

    pub fn is_push_only(&self) -> bool {
        unsafe { kernel_script_sig_is_push_only(self.inner) }
    }

    pub fn is_empty(&self) -> bool {
        unsafe { kernel_script_sig_is_empty(self.inner) }
    }
}

#[derive(Debug, Clone)]
pub struct WitnessRef {
    inner: *const kernel_TransactionWitness,
    marker: PhantomData<BlockRef>,
}

impl WitnessRef {
    pub fn stack_size(&self) -> usize {
        unsafe { kernel_witness_get_stack_size(self.inner) }
    }

    pub fn stack_item(&self, index: usize) -> Option<Vec<u8>> {
        let raw_item = unsafe { kernel_witness_get_stack_item(self.inner, index) };
        let vec = unsafe {
            std::slice::from_raw_parts((*raw_item).data, (*raw_item).size.try_into().unwrap())
        }
        .to_vec();
        Some(vec)
    }

    pub fn is_null(&self) -> bool {
        unsafe { kernel_witness_is_null(self.inner) }
    }
}

#[derive(Debug, Clone)]
pub struct OutPointRef {
    inner: *const kernel_TransactionOutPoint,
    marker: PhantomData<BlockRef>,
}

impl OutPointRef {
    pub fn tx_id(&self) -> Hash {
        let hash = unsafe { kernel_transaction_out_point_get_hash(self.inner) };
        Hash {
            hash: unsafe { (&*hash).hash },
        }
    }

    pub fn index(&self) -> u32 {
        unsafe { kernel_transaction_out_point_get_index(self.inner) }
    }
}

#[derive(Debug, Clone)]
pub struct TxOutRef {
    inner: *const kernel_TransactionOutput,
    marker: PhantomData<BlockRef>,
}

impl TxOutRef {
    pub fn value(&self) -> i64 {
        unsafe {
            let mut_ptr = self.inner as *mut kernel_TransactionOutput;
            kernel_transaction_output_get_amount(mut_ptr)
        }
    }

    pub fn script_pubkey(&self) -> ScriptPubkeyRef {
        let script_pubkey_ptr = unsafe { kernel_transaction_output_get_script_pubkey(self.inner) };

        ScriptPubkeyRef {
            inner: script_pubkey_ptr,
            marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TxInRef {
    inner: *const kernel_TransactionInput,
    marker: PhantomData<BlockRef>,
}

impl TxInRef {
    pub fn out_point(&self) -> OutPointRef {
        let out_point = unsafe { kernel_transaction_input_get_out_point(self.inner) };

        OutPointRef {
            inner: out_point,
            marker: PhantomData,
        }
    }

    pub fn script_sig(&self) -> ScriptSigRef {
        let script_sig = unsafe { kernel_transaction_input_get_script_sig(self.inner) };

        ScriptSigRef {
            inner: script_sig,
            marker: PhantomData,
        }
    }

    pub fn n_sequence(&self) -> u32 {
        unsafe { kernel_transaction_input_get_n_sequence(self.inner) }
    }

    pub fn witness(&self) -> WitnessRef {
        let witness = unsafe { kernel_transaction_input_get_witness(self.inner) };

        WitnessRef {
            inner: witness,
            marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionRef {
    inner: *const kernel_Transaction,
    marker: PhantomData<BlockRef>,
}

impl TransactionRef {
    pub fn hash(&self) -> Hash {
        let hash = unsafe { kernel_transaction_get_hash(self.inner) };
        Hash {
            hash: unsafe { (&*hash).hash },
        }
    }

    pub fn input_count(&self) -> usize {
        let count = unsafe { kernel_transaction_get_input_count(self.inner) };
        count as usize
    }

    pub fn input(&self, index: usize) -> Option<TxInRef> {
        let input = unsafe { kernel_transaction_get_input(self.inner, index) };
        if input.is_null() {
            return None;
        }

        Some(TxInRef {
            inner: input,
            marker: PhantomData,
        })
    }

    pub fn output_count(&self) -> usize {
        let count = unsafe { kernel_transaction_get_output_count(self.inner) };
        count as usize
    }

    pub fn output(&self, index: usize) -> Option<TxOutRef> {
        let output = unsafe { kernel_transaction_get_output(self.inner, index) };
        if output.is_null() {
            return None;
        }

        Some(TxOutRef {
            inner: output,
            marker: PhantomData,
        })
    }

    pub fn is_null(&self) -> bool {
        unsafe { kernel_transaction_is_null(self.inner) }
    }

    pub fn witness_hash(&self) -> Hash {
        let hash = unsafe { kernel_transaction_get_witness_hash(self.inner) };
        Hash {
            hash: unsafe { (&*hash).hash },
        }
    }

    pub fn value_out(&self) -> i64 {
        unsafe { kernel_transaction_get_value_out(self.inner) }
    }

    pub fn total_size(&self) -> usize {
        unsafe { kernel_transaction_get_total_size(self.inner) }
    }

    pub fn is_coinbase(&self) -> bool {
        unsafe { kernel_transaction_is_coinbase(self.inner) }
    }

    pub fn has_witness(&self) -> bool {
        unsafe { kernel_transaction_has_witness(self.inner) }
    }
}

#[derive(Debug, Clone)]
pub struct BlockRef {
    inner: *const kernel_BlockPointer,
}

impl BlockRef {
    pub fn hash(&self) -> Hash {
        let hash = unsafe { kernel_block_pointer_get_hash(self.inner) };
        let res = Hash {
            hash: unsafe { (*hash).hash },
        };
        unsafe { kernel_block_hash_destroy(hash) };
        res
    }

    pub fn transaction_count(&self) -> usize {
        let count = unsafe { kernel_block_pointer_get_transaction_count(self.inner) };
        count as usize
    }

    pub fn transaction(&self, index: usize) -> Option<TransactionRef> {
        let transaction = unsafe { kernel_block_pointer_get_transaction(self.inner, index) };
        if transaction.is_null() {
            return None;
        }

        Some(TransactionRef {
            inner: transaction,
            marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BlockUndoRef {
    inner: *const kernel_BlockUndo,
}

impl BlockUndoRef {
    pub fn transaction_count(&self) -> u64 {
        unsafe { kernel_block_undo_size(self.inner) }
    }

    pub fn transaction_undo_size(&self, transaction_index: u64) -> u64 {
        unsafe { kernel_block_undo_get_transaction_undo_size(self.inner, transaction_index) }
    }

    pub fn prevout_height_by_index(
        &self,
        transaction_index: u64,
        output_index: u64,
    ) -> Result<u32, BlockReaderError> {
        let height = unsafe {
            kernel_block_undo_get_transaction_output_height_by_index(
                self.inner,
                transaction_index,
                output_index,
            )
        };

        if height == 0 {
            return Err(BlockReaderError::OutOfBounds);
        }
        Ok(height)
    }

    pub fn prevout_by_index(&self, transaction_index: u64, prevout_index: u64) -> Option<TxOutRef> {
        let prev_out = unsafe {
            kernel_block_undo_copy_transaction_output_by_index(
                // TODO: implement non copy version
                // for blockreader
                self.inner,
                transaction_index,
                prevout_index,
            )
        };

        if prev_out.is_null() {
            None
        } else {
            Some(TxOutRef {
                inner: prev_out,
                marker: PhantomData,
            })
        }
    }
}

pub struct BlockReader {
    inner: *mut kernel_blockreader_Reader,
}

impl BlockReader {
    pub fn new(datadir: &str, chain_type: ChainType) -> Result<Arc<Self>, BlockReaderError> {
        let datadir_cstr =
            CString::new(datadir).map_err(|e| BlockReaderError::InvalidPath(e.to_string()))?;

        let chain_params = ChainParams::new(chain_type);

        let inner = unsafe {
            kernel_blockreader_create(chain_params.inner, datadir_cstr.as_ptr(), datadir.len())
        };

        if inner.is_null() {
            return Err(BlockReaderError::Internal(
                "Failed to create blockreader instance".to_string(),
            ));
        }

        Ok(Arc::new(BlockReader { inner }))
    }

    pub fn refresh(&self) {
        unsafe {
            kernel_blockreader_refresh(self.inner);
        }
    }

    pub fn ibd_status(&self) -> IBDStatus {
        unsafe {
            let status = kernel_blockreader_get_ibd_status(self.inner);
            status.into()
        }
    }

    pub fn best_validated_block_index(self: &Arc<Self>) -> Option<BlockReaderIndex> {
        unsafe {
            let ptr = kernel_blockreader_get_best_block_index(self.inner);
            BlockReaderIndex::from_raw_ptr(ptr, Arc::clone(self))
        }
    }

    pub fn block_index_at(self: &Arc<Self>, height: i32) -> Option<BlockReaderIndex> {
        unsafe {
            let ptr = kernel_blockreader_get_block_index_by_height(self.inner, height);
            BlockReaderIndex::from_raw_ptr(ptr, Arc::clone(self))
        }
    }
}

impl Drop for BlockReader {
    fn drop(&mut self) {
        unsafe {
            kernel_blockreader_destroy(self.inner);
        }
    }
}

unsafe impl Send for BlockReader {}
unsafe impl Sync for BlockReader {}

#[derive(Clone)]
pub struct BlockReaderIndex {
    inner: *const kernel_BlockIndex,
    reader: Arc<BlockReader>,
}

impl BlockReaderIndex {
    pub(crate) unsafe fn from_raw_ptr(
        ptr: *const kernel_BlockIndex,
        reader: Arc<BlockReader>,
    ) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(BlockReaderIndex { inner: ptr, reader })
        }
    }

    pub fn is_on_best_chain(&self) -> bool {
        unsafe { kernel_block_index_is_on_best_chain(self.reader.inner, self.inner) }
    }

    pub fn height(&self) -> i32 {
        unsafe { kernel_block_index_get_height(self.inner) }
    }

    pub fn block_hash(&self) -> Hash {
        unsafe {
            let hash_ptr = kernel_block_index_get_block_hash(self.inner);
            let result = Hash {
                hash: (*hash_ptr).hash,
            };
            kernel_block_hash_destroy(hash_ptr);
            result
        }
    }

    pub fn timestamp(&self) -> u32 {
        unsafe { kernel_block_index_get_timestamp(self.inner) }
    }

    pub fn version(&self) -> u32 {
        unsafe { kernel_block_index_get_version(self.inner) }
    }

    pub fn merkle_root(&self) -> Hash {
        unsafe {
            let merkle_root = kernel_block_index_get_merkle_root(self.inner);
            Hash {
                hash: (*merkle_root).hash,
            }
        }
    }

    pub fn bits(&self) -> u32 {
        unsafe { kernel_block_index_get_bits(self.inner) }
    }

    pub fn nonce(&self) -> u32 {
        unsafe { kernel_block_index_get_nonce(self.inner) }
    }

    pub fn median_time_past(&self) -> u32 {
        unsafe { kernel_block_index_get_median_time_past(self.inner) }
    }

    pub fn has_block_data(&self) -> bool {
        unsafe { kernel_block_index_has_block_data(self.inner) }
    }

    pub fn has_undo_data(&self) -> bool {
        unsafe { kernel_block_index_has_undo_data(self.inner) }
    }

    pub fn has_valid_transactions(&self) -> bool {
        unsafe { kernel_block_index_has_valid_transactions(self.inner) }
    }

    pub fn has_valid_chain(&self) -> bool {
        unsafe { kernel_block_index_has_valid_chain(self.inner) }
    }

    pub fn has_valid_scripts(&self) -> bool {
        unsafe { kernel_block_index_has_valid_scripts(self.inner) }
    }

    pub fn is_failed(&self) -> bool {
        unsafe { kernel_block_index_is_failed(self.inner) }
    }

    pub fn has_witness(&self) -> bool {
        unsafe { kernel_block_index_has_witness(self.inner) }
    }

    pub fn raw_header(&self) -> Result<Vec<u8>, BlockReaderError> {
        unsafe {
            let byte_array = kernel_block_index_get_raw_header(self.inner);
            if byte_array.is_null() {
                return Err(BlockReaderError::Internal(
                    "Failed to get raw header".to_string(),
                ));
            }

            let header_data =
                std::slice::from_raw_parts((*byte_array).data, (*byte_array).size).to_vec();

            kernel_byte_array_destroy(byte_array);
            Ok(header_data)
        }
    }

    pub fn block(&self) -> Result<BlockRef, BlockReaderError> {
        unsafe {
            let block = kernel_blockreader_get_block_by_index(self.reader.inner, self.inner);
            if block.is_null() {
                return Err(BlockReaderError::BlockNotFound(
                    self.block_hash().display_order(),
                ));
            }

            Ok(BlockRef { inner: block })
        }
    }

    pub fn block_undo(&self) -> Result<BlockUndoRef, BlockReaderError> {
        unsafe {
            let block_undo = kernel_blockreader_get_undo_data(self.reader.inner, self.inner);
            Ok(BlockUndoRef { inner: block_undo })
        }
    }

    pub fn previous(&self) -> Option<BlockReaderIndex> {
        let inner = unsafe { kernel_block_index_get_previous(self.inner) };
        if inner.is_null() {
            return None;
        }
        Some(BlockReaderIndex {
            inner,
            reader: self.reader.clone(),
        })
    }

    pub fn next(&self) -> Option<BlockReaderIndex> {
        let inner = unsafe {
            kernel_blockreader_get_block_index_by_height(self.reader.inner, self.height() + 1)
        };
        if inner.is_null() {
            return None;
        }
        Some(BlockReaderIndex {
            inner,
            reader: self.reader.clone(),
        })
    }

    pub fn iter_backwards(self) -> BlockIndexIterator {
        BlockIndexIterator {
            current: Some(self),
            direction: IterDirection::Backwards,
        }
    }

    pub fn iter_forwards(self) -> BlockIndexIterator {
        BlockIndexIterator {
            current: Some(self),
            direction: IterDirection::Forwards,
        }
    }

    pub fn iter_backwards_while<F>(self, predicate: F) -> ConditionalBlockIndexIterator<F>
    where
        F: FnMut(&BlockReaderIndex) -> bool,
    {
        ConditionalBlockIndexIterator {
            inner: self.iter_backwards(),
            predicate,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum IterDirection {
    Backwards,
    Forwards,
}

pub struct BlockIndexIterator {
    current: Option<BlockReaderIndex>,
    direction: IterDirection,
}

impl Iterator for BlockIndexIterator {
    type Item = BlockReaderIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(current_index) => match self.direction {
                IterDirection::Backwards => {
                    self.current = current_index.previous();
                    Some(current_index)
                }
                IterDirection::Forwards => {
                    self.current = current_index.next();
                    Some(current_index)
                }
            },
            None => None,
        }
    }
}

pub struct ConditionalBlockIndexIterator<F> {
    inner: BlockIndexIterator,
    predicate: F,
}

impl<F> Iterator for ConditionalBlockIndexIterator<F>
where
    F: FnMut(&BlockReaderIndex) -> bool,
{
    type Item = BlockReaderIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(index) => {
                if (self.predicate)(&index) {
                    Some(index)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

unsafe impl Send for BlockReaderIndex {}
unsafe impl Sync for BlockReaderIndex {}
