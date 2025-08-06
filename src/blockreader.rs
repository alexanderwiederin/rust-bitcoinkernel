use std::{ffi::CString, sync::Arc};

use libbitcoinkernel_sys::*;

use crate::{BlockRef, BlockUndoRef, ChainParams, ChainType, Hash};

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

#[derive(Debug)]
pub enum BlockIdentifier {
    Height(i32),
    Hash(Hash),
}

#[derive(Debug)]
pub enum BlockReaderError {
    Internal(String),
    InvalidPath(String),
    BlockNotFound(BlockIdentifier),
    ReadError(i32),
    ChainParamsError(String),
    TransactionIndexOutOfRange(i32, usize),
}

impl std::fmt::Display for BlockReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockReaderError::Internal(msg) => write!(f, "Internal error: {}", msg),
            BlockReaderError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            BlockReaderError::BlockNotFound(identifier) => match identifier {
                BlockIdentifier::Height(height) => {
                    write!(f, "Block not found at height: {}", height)
                }
                BlockIdentifier::Hash(hash) => {
                    write!(f, "Block not found with hash: {}", hash)
                }
            },
            BlockReaderError::ReadError(height) => write!(f, "Read error at height {}", height),
            BlockReaderError::ChainParamsError(msg) => write!(f, "Chain params error: {}", msg),
            BlockReaderError::TransactionIndexOutOfRange(height, index) => write!(
                f,
                "Transaction index {} out of range at height {}",
                index, height
            ),
        }
    }
}

impl std::error::Error for BlockReaderError {}

pub struct BlockReaderIndex {
    inner: *const kernel_BlockIndex,
    reader: Arc<BlockReader>,
}

impl BlockReaderIndex {
    pub(crate) unsafe fn from_raw_borrowed(
        ptr: *const kernel_BlockIndex,
        reader: Arc<BlockReader>,
    ) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(BlockReaderIndex { inner: ptr, reader })
        }
    }

    /* Basic Metadata */

    pub fn height(&self) -> i32 {
        unsafe { kernel_block_index_get_height(self.inner) }
    }

    pub fn block_hash(&self) -> Hash {
        unsafe {
            let hash_ptr = kernel_block_index_get_block_hash(self.inner);
            let result = Hash {
                hash: (&*hash_ptr).hash,
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
            let result = Hash {
                hash: (*merkle_root).hash,
            };
            result
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

    /* Block Status */

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

    /* Block Header */

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

    /* Block */

    pub fn block(&self) -> Result<BlockRef, BlockReaderError> {
        unsafe {
            let block = kernel_blockreader_get_block_by_index(self.reader.inner, self.inner);
            if block.is_null() {
                return Err(BlockReaderError::BlockNotFound(BlockIdentifier::Hash(
                    self.block_hash(),
                )));
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

    /* Chain Navigation */

    pub fn previous(&self) -> Result<BlockReaderIndex, BlockReaderError> {
        let inner = unsafe { kernel_get_previous_block_index(self.inner) };
        if inner.is_null() {
            return Err(BlockReaderError::BlockNotFound(BlockIdentifier::Height(
                self.height() - 1,
            )));
        }
        return Ok(BlockReaderIndex {
            inner,
            reader: self.reader.clone(),
        });
    }
}

unsafe impl Send for BlockReaderIndex {}
unsafe impl Sync for BlockReaderIndex {}

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

    pub fn best_validated_block(self: &Arc<Self>) -> Option<BlockReaderIndex> {
        unsafe {
            let ptr = kernel_blockreader_get_best_block_index(self.inner);
            BlockReaderIndex::from_raw_borrowed(ptr, Arc::clone(self))
        }
    }
}

impl Drop for BlockReader {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                kernel_blockreader_destroy(self.inner);
            }
        }
    }
}
