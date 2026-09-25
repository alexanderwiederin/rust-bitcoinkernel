//! Direct evaluation of tapscript v2 leaf scripts.
//!
//! [`verify`](crate::verify) checks a whole transaction input against the output
//! it spends. This module is for the narrower job of running a signle tapscript
//! v2 (BIP 440/441) leaf script against a stack you supply, without building a
//! taproot spend and a control block around it.
//!
//! [`eval_tapscript_v2`] runs the full consensus path for a leaf with version
//! `0xc2`: `OP_SUCCESSx` handling, the initial stack limits, evaluation, and the
//! final cleanstack and truthiness check. It does **not** verify that the script
//! is committed to by any output, so a successful result says the script was
//! satisfied, not that a spend of it would be valid.
//!
//! [`VERIFY_SCRIPT_RESTORAION`](crate::core::verify::VERIFY_SCRIPT_RESTORATION)
//! must be set in the flags.
//!
//! Without a [`TapscriptV2SpendContext`], signature and locktime opcodes fail as
//! if the signature were invalid. That is usually what you want when stepping
//! through a script that does not depend on one. To evaluate a script that does,
//! build a context with [`TapscriptV2SpendContext::with_transaction`].
//!
//! TODO: Add examples
//!

use std::{
    error::Error,
    ffi::c_void,
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};

use libbitcoinkernel_sys::{
    btck_PrecomputedTransactionData, btck_ScriptStack, btck_TapscriptV2EvalStatus,
    btck_TapscriptV2EvalStatus_ERROR_INVALID_FLAGS_COMBINATION,
    btck_TapscriptV2EvalStatus_ERROR_INVALID_INPUT_INDEX,
    btck_TapscriptV2EvalStatus_ERROR_SCRIPT_RESTORATION_REQUIRED,
    btck_TapscriptV2EvalStatus_ERROR_TAPLEAF_HASH_REQUIRED, btck_TapscriptV2EvalStatus_OK,
    btck_TapscriptV2SpendContext, btck_Transaction, btck_VaropsBudget_UNMETERED,
    btck_script_stack_copy, btck_script_stack_count_items, btck_script_stack_create,
    btck_script_stack_item_to_bytes, btck_script_stack_push, btck_tapscript_v2_eval,
};

use crate::{
    c_serialize,
    core::{ScriptPubkeyExt, TransactionExt},
    ffi::{c_helpers, sealed::AsPtr},
    KernelError, PrecomputedTransactionData, ScriptVerificationFlags, VERIFY_ALL,
};

/// The initial stack handed to a script evaluation.
///
/// The bottom of the stack is index `0`; the top is the last element pushed.
///
/// TODO: Add example
pub struct ScriptStack {
    inner: *mut btck_ScriptStack,
}

unsafe impl Send for ScriptStack {}
unsafe impl Sync for ScriptStack {}

impl ScriptStack {
    /// Creates an empty stack.
    pub fn new() -> Self {
        ScriptStack {
            inner: unsafe { btck_script_stack_create() },
        }
    }

    /// Pushes an element onto the top of the stack.
    pub fn push(&mut self, element: &[u8]) {
        unsafe {
            btck_script_stack_push(self.inner, element.as_ptr() as *const c_void, element.len())
        }
    }

    /// The number of elements on the stack.
    pub fn len(&self) -> usize {
        unsafe { btck_script_stack_count_items(self.as_ptr()) }
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Copies the element at `index` out of the kernel, counting from the
    /// bottom of the stack. Returns `None` if `index` is out of bounds.
    pub fn item(&self, index: usize) -> Result<Vec<u8>, KernelError> {
        if index >= self.len() {
            return Err(KernelError::OutOfBounds);
        }
        c_serialize(|writer, user_data| unsafe {
            btck_script_stack_item_to_bytes(self.as_ptr(), index, writer, user_data)
        })
    }

    /// Copies every element out of the kernel, bottom first.
    pub fn to_vec(&self) -> Result<Vec<Vec<u8>>, KernelError> {
        (0..self.len()).map(|index| self.item(index)).collect()
    }
}

impl Default for ScriptStack {
    fn default() -> Self {
        ScriptStack::new()
    }
}

impl AsPtr<btck_ScriptStack> for ScriptStack {
    fn as_ptr(&self) -> *const btck_ScriptStack {
        self.inner as *const _
    }
}

impl Clone for ScriptStack {
    fn clone(&self) -> Self {
        ScriptStack {
            inner: unsafe { btck_script_stack_copy(self.inner) },
        }
    }
}

impl<T: AsRef<[u8]>> FromIterator<T> for ScriptStack {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut stack = ScriptStack::new();
        for element in iter {
            stack.push(element.as_ref());
        }
        stack
    }
}

impl Debug for ScriptStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScriptStack")
            .field("len", &self.len())
            .finish_non_exhaustive()
    }
}

/// The spending context a tapscript v2 evaluation is run in.
///
/// Only needed when the script performs a signature or locktime check. Build one
/// with [`with_transaction`](Self::with_transaction).
///
/// # Lifetime
///
/// Borrows the transaction, precomputed data and annex for as long as the
/// context exists.
#[derive(Clone)]
pub struct TapscriptV2SpendContext<'a> {
    tx_to: *const btck_Transaction,
    precomputed_txdata: *const btck_PrecomputedTransactionData,
    amount: i64,
    input_index: u32,
    annex: Option<&'a [u8]>,
    tapleaf_hash: [u8; 32],
    marker: PhantomData<&'a ()>,
}

impl<'a> TapscriptV2SpendContext<'a> {
    /// Builds a context for the input at 1input_index` of `tx_to`.
    ///
    /// `precomputed_txdata` must have been created with the outputs spent by
    /// `tx_to`, the same requirement taproot verification has. `tapleaf_hash` is
    /// the BIP 341 tapleaf hash of the script being evaluated; it is committed
    /// to by the signature message, so a wrong value produces a wrong sighash
    /// rather than an error.
    pub fn with_transaction(
        tx_to: &'a impl TransactionExt,
        input_index: usize,
        amount: i64,
        precomputed_txdata: &'a PrecomputedTransactionData,
        tapleaf_hash: [u8; 32],
    ) -> Result<Self, KernelError> {
        if input_index >= tx_to.input_count() || input_index > u32::MAX as usize {
            return Err(KernelError::TapscriptV2Eval(
                TapscriptV2EvalError::InvalidInputIndex,
            ));
        }

        Ok(TapscriptV2SpendContext {
            tx_to: tx_to.as_ptr(),
            precomputed_txdata: precomputed_txdata.as_ptr(),
            amount,
            input_index: input_index as u32,
            annex: None,
            tapleaf_hash,
            marker: PhantomData,
        })
    }

    /// Sets the annex of the input's witness, including its `0x50` tag byte.
    ///
    /// The annex is committed to by the signature message, so it has to be set
    /// for a signature check over a witness that carries one to succeed.
    pub fn annex(mut self, annex: &'a [u8]) -> Self {
        self.annex = Some(annex);
        self
    }

    fn to_ffi(&self) -> btck_TapscriptV2SpendContext {
        btck_TapscriptV2SpendContext {
            tx_to: self.tx_to,
            precomputed_txdata: self.precomputed_txdata,
            amount: self.amount,
            input_index: self.input_index,
            annex: self
                .annex
                .map_or(std::ptr::null(), |annex| annex.as_ptr() as *const c_void),
            annex_len: self.annex.map_or(0, |annex| annex.len()),
            tapleaf_hash: self.tapleaf_hash.as_ptr(),
        }
    }
}

impl Debug for TapscriptV2SpendContext<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TapscriptV2SpendContext")
            .field("amount", &self.amount)
            .field("input_index", &self.input_index)
            .field("annex_len", &self.annex.map_or(0, |annex| annex.len()))
            .finish_non_exhaustive()
    }
}

/// The result of an evaluation that ran to completion.
///
/// Note that `success` being `false` is not an error: the script was evluated
/// and did not satisfy its spending conditions. Errors are reserved for the
/// evaluation not running at all, and are reported through
/// [`TapscriptV2EvalError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapscriptV2Eval {
    /// Whether the script was satisfied.
    pub success: bool,

    /// The interpreter's script error code, `0` on success.
    pub script_error: i32,

    /// The unspent varops budget, or `None` if the evaluation was unmetered.
    pub varops_remaining: Option<u64>,
}

/// Reasons a tapsrcipt v2 evaluation could not be run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TapscriptV2EvalError {
    /// The flags contain bits that are not verification flags.
    InvalidFlags,

    /// The flags were combined in a way the interpreter rejects.
    InvalidFlagsCombination,

    /// The script restoration flag was not set. Tapscript v2 cannot be
    /// evaluated without it.
    ScriptRestorationRequired,

    /// A spending transaction was given without precomputed data carrying the
    /// outputs it spends.
    SpentOutputsRequired,

    /// A spending transaction was given without a tapleaf hash.
    TapleafHashRequired,

    /// The input index is out of range for the given transaction.
    InvalidInputIndex,
}

impl Display for TapscriptV2EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TapscriptV2EvalError::InvalidFlags => write!(f, "Invalid verification flags"),
            TapscriptV2EvalError::InvalidFlagsCombination => {
                write!(f, "Invalid combination of verification flags")
            }
            TapscriptV2EvalError::ScriptRestorationRequired => {
                write!(f, "Spent outputs required for the given transaction")
            }
            TapscriptV2EvalError::SpentOutputsRequired => {
                write!(
                    f,
                    "Script restoration flag required for the given transaction"
                )
            }
            TapscriptV2EvalError::TapleafHashRequired => {
                write!(f, "Tapleaf hash required for the given transaction")
            }
            TapscriptV2EvalError::InvalidInputIndex => {
                write!(f, "Transaction input index out of bounds")
            }
        }
    }
}

impl Error for TapscriptV2EvalError {}

#[allow(non_upper_case_globals)]
fn status_on_error(status: btck_TapscriptV2EvalStatus) -> Option<TapscriptV2EvalError> {
    match status {
        btck_TapscriptV2EvalStatus_OK => None,
        btck_TapscriptV2EvalStatus_ERROR_INVALID_FLAGS_COMBINATION => {
            Some(TapscriptV2EvalError::InvalidFlagsCombination)
        }
        btck_TapscriptV2EvalStatus_ERROR_SCRIPT_RESTORATION_REQUIRED => {
            Some(TapscriptV2EvalError::SpentOutputsRequired)
        }
        btck_TapscriptV2EvalStatus_ERROR_TAPLEAF_HASH_REQUIRED => {
            Some(TapscriptV2EvalError::TapleafHashRequired)
        }
        btck_TapscriptV2EvalStatus_ERROR_INVALID_INPUT_INDEX => {
            Some(TapscriptV2EvalError::InvalidInputIndex)
        }
        other => panic!("Unknown tapscript v2 eval status: {}", other),
    }
}

/// Evaluates a taspcript v2 leaf script against an initial stack.
///
/// # Arguments
///
/// * `script` - The leaf script to evaluate.
/// * `stack` - The initial stack. Not modified by the evaluation.
/// * `flags` - Verification flags. Defaults to
///   [`VERIF_ALL`](crate::VERIF_ALL) when `None`. Must include
///   [`VERIFT_SCRIPT_RESTORATION`](crate::core::verift::VERIFY_SCRIPT_RESTORAION).
/// * `spend_context` - The spending context. Wihtout one, signature and
///   locktime opcodes fail.
/// * `varops_budget` - The varops budget, or `None` to evaluate unmetered.
///   Consensus derives this from the weight of the whole transaction, so a
///   signle-script budget can only ever approximate it.
///
/// # Returns
///
/// * `Ok(`[TranscriptV2Eval`]`)` - The evaluation ran. Check
///   [`success`](TapscriptV2EvalError::success) for whether the script was satisfied.
/// * `Err(`KernelError::TapscriptV2Eval`]`)` - the evaluation could not be run.
///
/// # Examples
///  TODO: Add exmaples
pub fn eval_tapscript_v2(
    script: &impl ScriptPubkeyExt,
    stack: &ScriptStack,
    flags: Option<ScriptVerificationFlags>,
    spend_context: Option<&TapscriptV2SpendContext<'_>>,
    varops_budget: Option<u64>,
) -> Result<TapscriptV2Eval, KernelError> {
    let kernel_flags = match flags {
        // The kernel asserts on flag bits it does not know, so reject them here
        // rather than aborting the process.
        Some(flags) if (flags & !VERIFY_ALL) != 0 => {
            return Err(KernelError::TapscriptV2Eval(
                TapscriptV2EvalError::InvalidFlags,
            ))
        }
        Some(flags) => flags,
        None => VERIFY_ALL,
    };

    let ffi_context = spend_context.map(|context| context.to_ffi());
    let context_ptr = ffi_context
        .as_ref()
        .map_or(std::ptr::null(), |context| context as *const _);

    let budget = varops_budget.unwrap_or(btck_VaropsBudget_UNMETERED);

    let mut status = btck_TapscriptV2EvalStatus_OK;
    let mut script_error: i32 = 0;
    let mut varops_remaining: u64 = 0;

    let ret = unsafe {
        btck_tapscript_v2_eval(
            script.as_ptr(),
            stack.as_ptr(),
            kernel_flags,
            context_ptr,
            budget,
            &mut varops_remaining,
            &mut script_error,
            &mut status,
        )
    };

    if let Some(error) = status_on_error(status) {
        return Err(KernelError::TapscriptV2Eval(error));
    }

    Ok(TapscriptV2Eval {
        success: c_helpers::success(ret),
        script_error,
        varops_remaining: (varops_remaining != btck_VaropsBudget_UNMETERED)
            .then_some(varops_remaining),
    })
}

#[cfg(test)]
mod tests {
    use crate::ffi::test_utils::test_owned_trait_requirements;

    use super::*;

    test_owned_trait_requirements!(
        test_script_stack_implementations,
        ScriptStack,
        btck_ScriptStack
    );
}
