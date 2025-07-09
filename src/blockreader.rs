use std::{ffi::CString, fmt};

use libbitcoinkernel_sys::*;

use crate::{BlockHash, ChainParams, ChainType};

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
    Hash(BlockHash),
}

#[derive(Debug)]
pub enum BlockReaderError {
    Internal(String),
    InvalidPath(String),
    BlockNotFound(BlockIdentifier),
    ReadError(i32),
    ChainParamsError(String),
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
        }
    }
}

impl std::error::Error for BlockReaderError {}

pub struct BlockReaderIndex {
    inner: *mut kernel_BlockIndex,
}

impl BlockReaderIndex {
    pub(crate) unsafe fn from_raw_borrowed(ptr: *mut kernel_BlockIndex) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(BlockReaderIndex { inner: ptr })
        }
    }

    pub fn height(&self) -> i32 {
        unsafe { kernel_block_index_get_height(self.inner) }
    }

    pub fn block_hash(&self) -> BlockHash {
        unsafe {
            let hash_ptr = kernel_block_index_get_block_hash(self.inner);
            let result = BlockHash {
                hash: (*hash_ptr).hash,
            };
            kernel_block_hash_destroy(hash_ptr);
            result
        }
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

    pub fn timestamp(&self) -> u32 {
        unsafe { kernel_block_index_get_timestamp(self.inner) }
    }

    pub fn transaction_count(&self) -> u32 {
        unsafe { kernel_Block_index_get_transaction_count(self.inner) }
    }
}

unsafe impl Send for BlockReaderIndex {}
unsafe impl Sync for BlockReaderIndex {}

pub struct BlockRef {
    inner: *mut kernel_Block,
}

unsafe impl Send for BlockRef {}
unsafe impl Sync for BlockRef {}

impl BlockRef {
    pub fn get_hash(&self) -> BlockHash {
        unsafe {
            let hash_ptr = kernel_blockreader_block_get_hash(self.inner);
            let result = BlockHash {
                hash: (*hash_ptr).hash,
            };
            kernel_block_hash_destroy(hash_ptr);
            result
        }
    }
}

impl From<BlockRef> for Vec<u8> {
    fn from(value: BlockRef) -> Self {
        let raw_block = unsafe { kernel_copy_block_data(value.inner) };
        let vec =
            unsafe { std::slice::from_raw_parts((*raw_block).data, (*raw_block).size) }.to_vec();

        unsafe { kernel_byte_array_destroy(raw_block) };
        vec
    }
}

impl Drop for BlockRef {
    fn drop(&mut self) {
        unsafe { kernel_blockreader_block_destroy(self.inner) };
    }
}

impl fmt::Debug for BlockRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.inner.is_null() {
            write!(f, "BlockRef(null)")
        } else {
            write!(f, "BlockRef(ptr: {:p})", self.inner)
        }
    }
}

pub struct BlockReader {
    inner: *mut kernel_blockreader_Reader,
}

impl BlockReader {
    pub fn new(datadir: &str, chain_type: ChainType) -> Result<Self, BlockReaderError> {
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

        Ok(BlockReader { inner })
    }

    pub fn refresh(&mut self) {
        unsafe {
            kernel_blockreader_refresh(self.inner);
        }
    }

    pub fn get_ibd_status(&self) -> IBDStatus {
        unsafe {
            let status = kernel_blockreader_get_ibd_status(self.inner);
            status.into()
        }
    }

    pub fn get_chain_height(&self) -> i32 {
        unsafe {
            let ptr = kernel_blockreader_get_best_validated_block(self.inner);
            if ptr.is_null() {
                return 0;
            }

            kernel_block_index_get_height(ptr)
        }
    }

    pub fn has_block(&self, height: i32) -> bool {
        if height < 0 {
            return false;
        }
        unsafe {
            let ptr = kernel_blockreader_get_block_index_by_height(self.inner, height);

            !ptr.is_null()
        }
    }

    pub fn get_block_hash(&self, height: i32) -> Result<BlockHash, BlockReaderError> {
        if height < 0 {
            return Err(BlockReaderError::ReadError(height));
        }

        unsafe {
            let block_index = kernel_blockreader_get_block_index_by_height(self.inner, height);
            if block_index.is_null() {
                return Err(BlockReaderError::BlockNotFound(BlockIdentifier::Height(
                    height,
                )));
            }

            let hash_ptr = kernel_block_index_get_block_hash(block_index);
            if hash_ptr.is_null() {
                return Err(BlockReaderError::Internal(
                    "Failed to get block hash".to_string(),
                ));
            }

            let hash_data = (*hash_ptr).hash;

            kernel_block_hash_destroy(hash_ptr);

            Ok(BlockHash { hash: hash_data })
        }
    }

    pub fn get_block(&self, height: i32) -> Result<BlockRef, BlockReaderError> {
        if height < 0 {
            return Err(BlockReaderError::ReadError(height));
        }

        unsafe {
            let block_ptr = kernel_blockreader_get_block_by_height(self.inner, height);
            if block_ptr.is_null() {
                return Err(BlockReaderError::BlockNotFound(BlockIdentifier::Height(
                    height,
                )));
            }

            Ok(BlockRef { inner: block_ptr })
        }
    }

    pub fn get_best_validated_block(&self) -> Option<BlockReaderIndex> {
        unsafe {
            let ptr = kernel_blockreader_get_best_validated_block(self.inner);
            BlockReaderIndex::from_raw_borrowed(ptr)
        }
    }

    pub fn get_block_index_by_hash(
        &self,
        hash: &BlockHash,
    ) -> Result<BlockReaderIndex, BlockReaderError> {
        let block_hash = kernel_BlockHash { hash: hash.hash };
        unsafe {
            let block_index = kernel_blockreader_get_block_index_by_hash(self.inner, &block_hash);
            if block_index.is_null() {
                Err(BlockReaderError::Internal("Block not found".to_string()))
            } else {
                Ok(BlockReaderIndex { inner: block_index })
            }
        }
    }

    pub fn get_block_by_hash(&self, hash: &BlockHash) -> Result<BlockRef, BlockReaderError> {
        let block_hash = kernel_BlockHash { hash: hash.hash };
        unsafe {
            let block_ptr = kernel_blockreader_get_block_by_hash(self.inner, &block_hash);
            if block_ptr.is_null() {
                return Err(BlockReaderError::Internal("Block not found".to_string()));
            }
            Ok(BlockRef { inner: block_ptr })
        }
    }

    pub fn get_genesis_hash(&self) -> Result<BlockHash, BlockReaderError> {
        unsafe {
            let hash_ptr = kernel_blockreader_get_genesis_hash(self.inner);
            if hash_ptr.is_null() {
                return Err(BlockReaderError::Internal(
                    "Failed to get genesis hash".to_string(),
                ));
            }

            let result = BlockHash {
                hash: (*hash_ptr).hash,
            };
            kernel_block_hash_destroy(hash_ptr);
            Ok(result)
        }
    }

    pub fn is_block_in_active_chain(&self, block_index: &BlockReaderIndex) -> bool {
        unsafe { kernel_blockreader_is_block_in_active_chain(self.inner, block_index.inner) }
    }

    pub fn get_block_index_by_height(
        &self,
        height: i32,
    ) -> Result<BlockReaderIndex, BlockReaderError> {
        if height < 0 {
            return Err(BlockReaderError::ReadError(height));
        }

        unsafe {
            let block_index = kernel_blockreader_get_block_index_by_height(self.inner, height);
            if block_index.is_null() {
                return Err(BlockReaderError::BlockNotFound(BlockIdentifier::Height(
                    height,
                )));
            }

            Ok(BlockReaderIndex { inner: block_index })
        }
    }

    pub fn get_block_header(&self, height: i32) -> Result<Vec<u8>, BlockReaderError> {
        let block_index = self.get_block_index_by_height(height)?;
        block_index.raw_header()
    }

    pub fn get_headers_raw(
        &self,
        start_height: i32,
        count: usize,
    ) -> Result<Vec<u8>, BlockReaderError> {
        unsafe {
            let batch = kernel_blockreader_get_headers_raw(self.inner, start_height, count);
            if batch.is_null() {
                return Err(BlockReaderError::Internal(
                    "Failed to get geaders batch".to_string(),
                ));
            }

            let data = std::slice::from_raw_parts((*batch).data, (*batch).size).to_vec();
            kernel_byte_array_destroy(batch);
            Ok(data)
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
