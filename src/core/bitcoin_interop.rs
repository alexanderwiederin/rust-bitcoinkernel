//! Zero-copy [`Deref`] bridge from [`ScriptPubkey`] and [`ScriptPubkeyRef`]
//! to [`bitcoin::Script`], enabled by the `bitcoin` feature flag.
//!
//! This allows kernel script types to transparently use the full
//! `bitcoin::Script` API without any allocation or copying.
//!
//! ```no_run
//! use bitcoinkernel::ScriptPubkey;
//!
//! let bytes = hex::decode("5120deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap();
//! let script = ScriptPubkey::new(&bytes).unwrap();
//!
//! assert!(script.is_p2tr());
//! assert!(script.witness_version().is_some());
//! assert!(!script.is_op_return());
//! println!("{}", script.to_asm_string());
//! ```

use bitcoin::Script;
use libbitcoinkernel_sys::btck_script_pubkey_to_bytes;
use std::{ffi::c_void, ops::Deref, panic};

use crate::core::script::{ScriptPubkey, ScriptPubkeyRef};
use crate::{c_helpers, ScriptPubkeyExt};

struct ScriptBytesOut {
    ptr: *const u8,
    len: usize,
}

// SAFETY: This function relies on `btck_script_pubkey_to_bytes` invoking
// the writer callback synchronously, before returning. The raw pointer
// captured in `ScriptBytesOut` is only valid for the duration of that
// callback. If the kernel implementation ever changed to call the writer
// asynchronously or defer it, this would be unsound.
unsafe fn script_as_bytes<T: ScriptPubkeyExt>(val: &T) -> &[u8] {
    unsafe extern "C" fn writer(data: *const c_void, len: usize, user_data: *mut c_void) -> i32 {
        panic::catch_unwind(|| {
            let out = &mut *(user_data as *mut ScriptBytesOut);
            out.ptr = data as *const u8;
            out.len = len;
            c_helpers::to_c_result(true)
        })
        .unwrap_or_else(|_| c_helpers::to_c_result(false))
    }

    let mut out = ScriptBytesOut {
        ptr: std::ptr::null(),
        len: 0,
    };
    let ret = btck_script_pubkey_to_bytes(
        val.as_ptr(),
        Some(writer),
        &mut out as *mut ScriptBytesOut as *mut c_void,
    );
    assert!(
        c_helpers::success(ret),
        "btck_script_pubkey_to_bytes should never fail for a valid ScriptPubkey"
    );

    if out.ptr.is_null() {
        return &[];
    }

    std::slice::from_raw_parts(out.ptr, out.len)
}

impl<'a> Deref for ScriptPubkeyRef<'a> {
    type Target = Script;

    fn deref(&self) -> &Script {
        let bytes = unsafe { script_as_bytes(self) };
        Script::from_bytes(bytes)
    }
}

impl Deref for ScriptPubkey {
    type Target = Script;

    fn deref(&self) -> &Script {
        let bytes = unsafe { script_as_bytes(self) };
        Script::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::ScriptPubkey;

    #[test]
    fn test_empty_script_via_deref() {
        let script = ScriptPubkey::new(&[]).unwrap();
        assert_eq!(script.as_bytes(), &[]);
    }

    #[test]
    fn test_is_p2tr_via_deref() {
        let p2tr =
            hex::decode("5120deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")
                .unwrap();
        let script = ScriptPubkey::new(&p2tr).unwrap();
        assert!(script.is_p2tr());
        assert!(!script.is_p2wpkh());
        assert!(!script.is_p2pkh());
    }

    #[test]
    fn test_is_p2pkh_via_deref() {
        let p2pkh = hex::decode("76a914deadbeefdeadbeefdeadbeefdeadbeefdeadbeef88ac").unwrap();
        let script = ScriptPubkey::new(&p2pkh).unwrap();
        assert!(script.is_p2pkh());
        assert!(!script.is_p2tr());
    }

    #[test]
    fn test_is_p2wpkh_via_deref() {
        let p2wpkh = hex::decode("0014deadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap();
        let script = ScriptPubkey::new(&p2wpkh).unwrap();
        assert!(script.is_p2wpkh());
        assert!(!script.is_p2tr());
    }

    #[test]
    fn test_script_ref_deref() {
        let p2tr =
            hex::decode("5120deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")
                .unwrap();
        let script = ScriptPubkey::new(&p2tr).unwrap();
        let script_ref = script.as_ref();
        assert!(script_ref.is_p2tr());
        assert_eq!(script.is_p2tr(), script_ref.is_p2tr());
    }

    #[test]
    fn test_full_script_api_via_deref() {
        let p2tr =
            hex::decode("5120deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")
                .unwrap();
        let script = ScriptPubkey::new(&p2tr).unwrap();
        assert!(script.is_p2tr());
        assert!(script.witness_version().is_some());
        assert!(!script.is_op_return());
        assert!(!script.is_p2sh());
        assert!(!script.is_p2wsh());
    }

    #[test]
    fn test_bytes_roundtrip() {
        let bytes =
            hex::decode("5120deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")
                .unwrap();
        let script = ScriptPubkey::new(&bytes).unwrap();
        assert_eq!(script.as_bytes(), bytes.as_slice());
    }
}
