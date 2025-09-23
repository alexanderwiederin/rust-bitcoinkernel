use std::{ffi::c_void, marker::PhantomData};

use libbitcoinkernel_sys::{
    btck_Transaction, btck_TransactionOutput, btck_transaction_copy, btck_transaction_count_inputs,
    btck_transaction_count_outputs, btck_transaction_create, btck_transaction_destroy,
    btck_transaction_get_output_at, btck_transaction_output_copy, btck_transaction_output_create,
    btck_transaction_output_destroy, btck_transaction_output_get_amount,
    btck_transaction_output_get_script_pubkey, btck_transaction_to_bytes,
};

use crate::{
    c_serialize,
    ffi::sealed::{AsPtr, FromMutPtr, FromPtr},
    KernelError, ScriptPubkeyExt,
};

use super::script::ScriptPubkeyRef;

/// Common operations for transactions, implemented by both owned and borrowed types.
pub trait TransactionExt: AsPtr<btck_Transaction> {
    /// Returns the number of outputs in this transaction.
    fn output_count(&self) -> usize {
        unsafe { btck_transaction_count_outputs(self.as_ptr()) as usize }
    }

    /// Returns a reference to the output at the specified index.
    ///
    /// # Arguments
    /// * `index` - The zero-based index of the output to retrieve
    ///
    /// # Returns
    /// * `Ok(RefType<TxOut, Transaction>)` - A reference to the output
    /// * `Err(KernelError::OutOfBounds)` - If the index is invalid
    fn output(&self, index: usize) -> Result<TxOutRef<'_>, KernelError> {
        if index >= self.output_count() {
            return Err(KernelError::OutOfBounds);
        }

        let tx_out_ref =
            unsafe { TxOutRef::from_ptr(btck_transaction_get_output_at(self.as_ptr(), index)) };

        Ok(tx_out_ref)
    }

    fn input_count(&self) -> usize {
        unsafe { btck_transaction_count_inputs(self.as_ptr()) as usize }
    }

    /// Consensus encodes the transaction to Bitcoin wire format.
    fn consensus_encode(&self) -> Result<Vec<u8>, KernelError> {
        c_serialize(|callback, user_data| unsafe {
            btck_transaction_to_bytes(self.as_ptr(), Some(callback), user_data)
        })
    }
}

/// A Bitcoin transaction.
pub struct Transaction {
    inner: *mut btck_Transaction,
}

unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

impl Transaction {
    pub fn new(transaction_bytes: &[u8]) -> Result<Self, KernelError> {
        let inner = unsafe {
            btck_transaction_create(
                transaction_bytes.as_ptr() as *const c_void,
                transaction_bytes.len(),
            )
        };

        if inner.is_null() {
            Err(KernelError::Internal(
                "Failed to create transaction from bytes".to_string(),
            ))
        } else {
            Ok(Transaction { inner })
        }
    }

    pub fn as_ref(&self) -> TransactionRef<'_> {
        unsafe { TransactionRef::from_ptr(self.inner as *const _) }
    }
}

impl AsPtr<btck_Transaction> for Transaction {
    fn as_ptr(&self) -> *const btck_Transaction {
        self.inner as *const _
    }
}

impl FromMutPtr<btck_Transaction> for Transaction {
    unsafe fn from_ptr(ptr: *mut btck_Transaction) -> Self {
        Transaction { inner: ptr }
    }
}

impl TransactionExt for Transaction {}

impl Clone for Transaction {
    fn clone(&self) -> Self {
        Transaction {
            inner: unsafe { btck_transaction_copy(self.inner) },
        }
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        unsafe { btck_transaction_destroy(self.inner) }
    }
}

impl TryFrom<&[u8]> for Transaction {
    type Error = KernelError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Transaction::new(bytes)
    }
}

impl TryFrom<Transaction> for Vec<u8> {
    type Error = KernelError;

    fn try_from(transaction: Transaction) -> Result<Self, Self::Error> {
        transaction.consensus_encode()
    }
}

impl TryFrom<&Transaction> for Vec<u8> {
    type Error = KernelError;

    fn try_from(transaction: &Transaction) -> Result<Self, Self::Error> {
        transaction.consensus_encode()
    }
}

pub struct TransactionRef<'a> {
    inner: *const btck_Transaction,
    marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for TransactionRef<'a> {}
unsafe impl<'a> Sync for TransactionRef<'a> {}

impl<'a> TransactionRef<'a> {
    pub fn to_owned(&self) -> Transaction {
        Transaction {
            inner: unsafe { btck_transaction_copy(self.inner) },
        }
    }
}

impl<'a> AsPtr<btck_Transaction> for TransactionRef<'a> {
    fn as_ptr(&self) -> *const btck_Transaction {
        self.inner
    }
}

impl<'a> FromPtr<btck_Transaction> for TransactionRef<'a> {
    unsafe fn from_ptr(ptr: *const btck_Transaction) -> Self {
        TransactionRef {
            inner: ptr,
            marker: PhantomData,
        }
    }
}
impl<'a> TransactionExt for TransactionRef<'a> {}

impl<'a> Clone for TransactionRef<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> Copy for TransactionRef<'a> {}

/// Common operations for transaction outputs, implemented by both owned and borrowed types.
pub trait TxOutExt: AsPtr<btck_TransactionOutput> {
    /// Returns the amount of this output in satoshis.
    fn value(&self) -> i64 {
        unsafe { btck_transaction_output_get_amount(self.as_ptr()) }
    }

    /// Returns a reference to the script pubkey that defines how this output can be spent.
    ///
    /// # Returns
    /// * `RefType<ScriptPubkey, TxOut>` - A reference to the script pubkey
    fn script_pubkey(&self) -> ScriptPubkeyRef<'_> {
        let ptr = unsafe { btck_transaction_output_get_script_pubkey(self.as_ptr()) };
        unsafe { ScriptPubkeyRef::from_ptr(ptr) }
    }
}

/// A single transaction output containing a value and spending conditions.
///
/// Transaction outputs can be created from a script pubkey and amount, or retrieved
/// from existing transactions. They represent spendable coins in the UTXO set.
#[derive(Debug)]
pub struct TxOut {
    inner: *mut btck_TransactionOutput,
}

unsafe impl Send for TxOut {}
unsafe impl Sync for TxOut {}

impl TxOut {
    /// Creates a new transaction output with the specified script and amount.
    ///
    /// # Arguments
    /// * `script_pubkey` - The script defining how this output can be spent
    /// * `amount` - The amount in satoshis
    pub fn new(script_pubkey: &impl ScriptPubkeyExt, amount: i64) -> Self {
        TxOut {
            inner: unsafe { btck_transaction_output_create(script_pubkey.as_ptr(), amount) },
        }
    }

    pub fn as_ref(&self) -> TxOutRef<'_> {
        unsafe { TxOutRef::from_ptr(self.inner as *const _) }
    }
}

impl AsPtr<btck_TransactionOutput> for TxOut {
    fn as_ptr(&self) -> *const btck_TransactionOutput {
        self.inner as *const _
    }
}

impl FromMutPtr<btck_TransactionOutput> for TxOut {
    unsafe fn from_ptr(ptr: *mut btck_TransactionOutput) -> Self {
        TxOut { inner: ptr }
    }
}

impl TxOutExt for TxOut {}

impl Clone for TxOut {
    fn clone(&self) -> Self {
        TxOut {
            inner: unsafe { btck_transaction_output_copy(self.inner) },
        }
    }
}

impl Drop for TxOut {
    fn drop(&mut self) {
        unsafe { btck_transaction_output_destroy(self.inner) }
    }
}

pub struct TxOutRef<'a> {
    inner: *const btck_TransactionOutput,
    marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for TxOutRef<'a> {}
unsafe impl<'a> Sync for TxOutRef<'a> {}

impl<'a> TxOutRef<'a> {
    pub fn to_owned(&self) -> TxOut {
        TxOut {
            inner: unsafe { btck_transaction_output_copy(self.inner) },
        }
    }
}

impl<'a> AsPtr<btck_TransactionOutput> for TxOutRef<'a> {
    fn as_ptr(&self) -> *const btck_TransactionOutput {
        self.inner as *const _
    }
}

impl<'a> FromPtr<btck_TransactionOutput> for TxOutRef<'a> {
    unsafe fn from_ptr(ptr: *const btck_TransactionOutput) -> Self {
        TxOutRef {
            inner: ptr,
            marker: PhantomData,
        }
    }
}

impl<'a> TxOutExt for TxOutRef<'a> {}

impl<'a> Clone for TxOutRef<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> Copy for TxOutRef<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_utils::{test_owned_ffi_traits, test_ref_ffi_traits};
    use crate::core::ScriptPubkey;

    test_owned_ffi_traits!(
        test_transaction_implementations,
        Transaction,
        btck_Transaction
    );
    test_ref_ffi_traits!(
        test_transaction_ref_implementations,
        TransactionRef<'static>,
        btck_Transaction
    );

    test_owned_ffi_traits!(test_txout_implementations, TxOut, btck_TransactionOutput);
    test_ref_ffi_traits!(
        test_txout_ref_implementations,
        TxOutRef<'static>,
        btck_TransactionOutput
    );

    // Helper function to create valid transaction bytes
    fn create_test_transaction_bytes() -> Vec<u8> {
        // A simple valid Bitcoin transaction in hex format
        // This is a minimal transaction with 2 inputs and 3 outputs
        hex::decode(
            "0200000002f4f1c5c8e8d8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a1b2c3d4e5f6a7b80000000000fefffffffedc\
            ba9876543210fedcba9876543210fedcba9876543210fedcba98765432100000000000feffffff0300e1f50500000000160014\
            751e76e8199196d454941c45d1b3a323f1433bd600ca9a3b00000000160014ab68025513c3dbd2f7b92a94e0581f5d50f654e7\
            cd1d00000000160014d85c2b71d0060b09c9886aeb815e50991dda124d00000000"
        ).unwrap()
    }

    // Helper to create a test script pubkey
    fn create_test_script_pubkey() -> ScriptPubkey {
        // P2WPKH script: OP_0 <20-byte-hash>
        let script_bytes = hex::decode("0014751e76e8199196d454941c45d1b3a323f1433bd6").unwrap();
        ScriptPubkey::new(&script_bytes).unwrap()
    }

    #[test]
    fn test_transaction_new() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes);
        assert!(tx.is_ok());
    }

    #[test]
    fn test_transaction_new_invalid_bytes() {
        let invalid_bytes = vec![0x00, 0x01, 0x02];
        let tx = Transaction::new(&invalid_bytes);

        assert!(matches!(tx, Err(KernelError::Internal(_))));
    }

    #[test]
    fn test_transaction_output_count() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();

        let count = tx.output_count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_transaction_input_count() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();

        let count = tx.input_count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_transaction_get_output() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();

        let output = tx.output(0);
        assert!(output.is_ok());

        let tx_out = output.unwrap();
        assert_eq!(tx_out.value(), 100_000_000);
    }

    #[test]
    fn test_transaction_get_output_out_of_bounds() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();

        let output = tx.output(999);

        assert!(matches!(output, Err(KernelError::OutOfBounds)));
    }

    #[test]
    fn test_transaction_consensus_encode() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();

        let encoded = tx.consensus_encode();
        assert!(encoded.is_ok());

        let encoded_bytes = encoded.unwrap();
        assert_eq!(encoded_bytes, tx_bytes);
    }

    #[test]
    fn test_transaction_clone() {
        let tx_bytes = create_test_transaction_bytes();
        let tx1 = Transaction::new(&tx_bytes).unwrap();
        let tx2 = tx1.clone();

        assert_eq!(tx1.output_count(), tx2.output_count());
        assert_eq!(tx1.input_count(), tx2.input_count());
    }

    #[test]
    fn test_transaction_try_from_bytes() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::try_from(tx_bytes.as_slice());
        assert!(tx.is_ok());
    }

    #[test]
    fn test_transaction_to_vec() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();

        let vec_result = Vec::<u8>::try_from(tx);
        assert!(vec_result.is_ok());
        assert_eq!(vec_result.unwrap(), tx_bytes);
    }

    #[test]
    fn test_transaction_ref_to_vec() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();

        let vec_result = Vec::<u8>::try_from(&tx);
        assert!(vec_result.is_ok());
        assert_eq!(vec_result.unwrap(), tx_bytes);
    }

    #[test]
    fn test_transaction_ref_to_owned() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();
        let tx_ref = tx.as_ref();

        let owned = tx_ref.to_owned();
        assert_eq!(owned.output_count(), tx.output_count());
    }

    #[test]
    fn test_transaction_ref_copy() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();
        let tx_ref1 = tx.as_ref();
        let tx_ref2 = tx_ref1;

        assert_eq!(tx_ref1.output_count(), tx_ref2.output_count());
    }

    #[test]
    fn test_txout_new() {
        let script = create_test_script_pubkey();
        let amount = 100_000_000;

        let tx_out = TxOut::new(&script, amount);
        assert_eq!(tx_out.value(), amount);
    }

    #[test]
    fn test_txout_value() {
        let script = create_test_script_pubkey();
        let amount = 50_000_000;

        let tx_out = TxOut::new(&script, amount);
        assert_eq!(tx_out.value(), amount);
    }

    #[test]
    fn test_txout_clone() {
        let script = create_test_script_pubkey();
        let amount = 25_000_000;

        let tx_out1 = TxOut::new(&script, amount);
        let tx_out2 = tx_out1.clone();

        assert_eq!(tx_out1.value(), tx_out2.value());
    }

    #[test]
    fn test_txout_ref_to_owned() {
        let script = create_test_script_pubkey();
        let amount = 75_000_000;

        let tx_out = TxOut::new(&script, amount);
        let tx_out_ref = tx_out.as_ref();

        let owned = tx_out_ref.to_owned();
        assert_eq!(owned.value(), amount);
    }

    #[test]
    fn test_txout_ref_copy() {
        let script = create_test_script_pubkey();
        let amount = 10_000;

        let tx_out = TxOut::new(&script, amount);
        let ref1 = tx_out.as_ref();
        let ref2 = ref1;

        assert_eq!(ref1.value(), ref2.value());
    }

    #[test]
    fn test_transaction_multiple_outputs() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();

        let output_count = tx.output_count();
        for i in 0..output_count {
            let _output = tx.output(i).unwrap();
        }
    }

    #[test]
    fn test_transaction_from_mut_ptr() {
        let tx_bytes = create_test_transaction_bytes();
        let tx1 = Transaction::new(&tx_bytes).unwrap();

        let ptr = unsafe { btck_transaction_copy(tx1.as_ptr()) };

        let tx2 = unsafe { Transaction::from_ptr(ptr) };

        assert_eq!(tx1.output_count(), tx2.output_count());
        assert_eq!(tx1.input_count(), tx2.input_count());
    }

    #[test]
    fn test_transaction_ref_from_ptr() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();

        let tx_ref = unsafe { TransactionRef::from_ptr(tx.as_ptr()) };

        assert_eq!(tx.output_count(), tx_ref.output_count());
        assert_eq!(tx.input_count(), tx_ref.input_count());
    }

    #[test]
    fn test_txout_from_mut_ptr() {
        let script = create_test_script_pubkey();
        let amount = 100_000_000;
        let txout1 = TxOut::new(&script, amount);

        let ptr = unsafe { btck_transaction_output_copy(txout1.as_ptr()) };

        let txout2 = unsafe { TxOut::from_ptr(ptr) };

        assert_eq!(txout1.value(), txout2.value());
    }

    #[test]
    fn test_txout_ref_from_ptr() {
        let script = create_test_script_pubkey();
        let amount = 50_000_000;
        let txout = TxOut::new(&script, amount);

        let txout_ref = unsafe { TxOutRef::from_ptr(txout.as_ptr()) };

        assert_eq!(txout.value(), txout_ref.value());
    }

    #[test]
    fn test_transaction_ref_clone() {
        let tx_bytes = create_test_transaction_bytes();
        let tx = Transaction::new(&tx_bytes).unwrap();
        let tx_ref1 = tx.as_ref();
        let tx_ref2 = tx_ref1.clone(); // Explicit clone call

        assert_eq!(tx_ref1.output_count(), tx_ref2.output_count());
    }

    #[test]
    fn test_txout_ref_clone() {
        let script = create_test_script_pubkey();
        let amount = 50_000_000;
        let tx_out = TxOut::new(&script, amount);
        let ref1 = tx_out.as_ref();
        let ref2 = ref1.clone(); // Explicit clone call

        assert_eq!(ref1.value(), ref2.value());
    }
}
