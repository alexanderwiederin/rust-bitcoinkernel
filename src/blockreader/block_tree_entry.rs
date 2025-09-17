use std::marker::PhantomData;

use libbitcoinkernel_sys::{
    btck_BlockTreeEntry, btck_block_hash_destroy, btck_block_tree_entry_get_block_hash,
    btck_block_tree_entry_get_height, btck_block_tree_entry_get_previous,
};

use crate::BlockHash;

use super::BlockReader;

#[derive(Debug)]
pub struct ReaderBlockTreeEntry<'a> {
    inner: *const btck_BlockTreeEntry,
    marker: PhantomData<&'a BlockReader>,
}

unsafe impl Send for ReaderBlockTreeEntry<'_> {}
unsafe impl Sync for ReaderBlockTreeEntry<'_> {}

impl<'a> ReaderBlockTreeEntry<'a> {
    pub unsafe fn from_ptr(ptr: *const btck_BlockTreeEntry) -> Self {
        ReaderBlockTreeEntry {
            inner: ptr,
            marker: PhantomData,
        }
    }

    /// Move to the previous entry in the block tree. E.g. from height n to
    /// height n-1.
    pub fn prev(self) -> Option<ReaderBlockTreeEntry<'a>> {
        let inner = unsafe { btck_block_tree_entry_get_previous(self.inner) };

        if inner.is_null() {
            return None;
        }

        Some(unsafe { ReaderBlockTreeEntry::from_ptr(inner) })
    }

    /// Returns the current height associated with this BlockTreeEntry.
    pub fn height(&self) -> i32 {
        unsafe { btck_block_tree_entry_get_height(self.inner) }
    }

    /// Returns the current block hash associated with this BlockTreeEntry.
    pub fn block_hash(&self) -> BlockHash {
        let hash = unsafe { btck_block_tree_entry_get_block_hash(self.inner) };
        let res = BlockHash {
            hash: unsafe { (&*hash).hash },
        };
        unsafe { btck_block_hash_destroy(hash) };
        res
    }

    pub fn as_ptr(&self) -> *const btck_BlockTreeEntry {
        self.inner
    }
}

impl<'a> Clone for ReaderBlockTreeEntry<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> Copy for ReaderBlockTreeEntry<'a> {}
