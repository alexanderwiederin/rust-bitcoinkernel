use std::ffi::CString;

use libbitcoinkernel_sys::{
    btck_BlockReader, btck_BlockReaderOptions, btck_blockreader_block_spent_outputs_read,
    btck_blockreader_create, btck_blockreader_destroy, btck_blockreader_get_validated_chain,
    btck_blockreader_options_create, btck_blockreader_options_destroy, btck_blockreader_read_block,
};

use crate::{Block, BlockSpentOutputs, Context, KernelError};

use super::{BlockReaderChain, ReaderBlockTreeEntry};

pub struct BlockReader {
    inner: *mut btck_BlockReader,
}

unsafe impl Send for BlockReader {}
unsafe impl Sync for BlockReader {}

impl BlockReader {
    pub fn new(chainman_opts: BlockReaderOptions) -> Result<Self, KernelError> {
        let inner = unsafe { btck_blockreader_create(chainman_opts.inner) };
        if inner.is_null() {
            return Err(KernelError::Internal(
                "Failed to create block reader.".to_string(),
            ));
        }
        Ok(Self { inner })
    }

    /// Read a block from disk by its block tree entry.
    pub fn read_block_data(&self, entry: &ReaderBlockTreeEntry) -> Result<Block, KernelError> {
        let inner = unsafe { btck_blockreader_read_block(self.inner, entry.as_ptr()) };
        if inner.is_null() {
            return Err(KernelError::Internal("Failed to read block.".to_string()));
        }
        Ok(unsafe { Block::from_ptr(inner) })
    }

    /// Read a block's spent outputs data from disk by its block tree entry.
    pub fn read_spent_outputs(
        &self,
        entry: &ReaderBlockTreeEntry,
    ) -> Result<BlockSpentOutputs, KernelError> {
        let inner =
            unsafe { btck_blockreader_block_spent_outputs_read(self.inner, entry.as_ptr()) };
        if inner.is_null() {
            return Err(KernelError::Internal(
                "Failed to read undo data.".to_string(),
            ));
        }
        Ok(unsafe { BlockSpentOutputs::from_ptr(inner) })
    }

    pub fn active_chain(&self) -> BlockReaderChain<'_> {
        let ptr = unsafe { btck_blockreader_get_validated_chain(self.inner) };
        unsafe { BlockReaderChain::from_ptr(ptr) }
    }
}

impl Drop for BlockReader {
    fn drop(&mut self) {
        unsafe {
            btck_blockreader_destroy(self.inner);
        }
    }
}

/// Holds the configuration options for creating a new [`ChainstateManager`]
pub struct BlockReaderOptions {
    inner: *mut btck_BlockReaderOptions,
}

impl BlockReaderOptions {
    /// Create a new option
    ///
    /// # Arguments
    /// * `context` -  The [`ChainstateManager`] for which these options are created has to use the same [`Context`].
    /// * `data_dir` - The directory into which the [`ChainstateManager`] will write its data.
    pub fn new(context: &Context, data_dir: &str, blocks_dir: &str) -> Result<Self, KernelError> {
        let c_data_dir = CString::new(data_dir)?;
        let c_blocks_dir = CString::new(blocks_dir)?;
        let inner = unsafe {
            btck_blockreader_options_create(
                context.as_ptr(),
                c_data_dir.as_ptr(),
                c_data_dir.as_bytes().len(),
                c_blocks_dir.as_ptr(),
                c_blocks_dir.as_bytes().len(),
            )
        };
        if inner.is_null() {
            return Err(KernelError::Internal(
                "Failed to create chainstate manager options.".to_string(),
            ));
        }
        Ok(Self { inner })
    }
}

impl Drop for BlockReaderOptions {
    fn drop(&mut self) {
        unsafe {
            btck_blockreader_options_destroy(self.inner);
        }
    }
}
