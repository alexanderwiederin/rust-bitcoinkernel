use std::{ffi::c_void, marker::PhantomData};

use libbitcoinkernel_sys::{
    btck_ScriptPubkey, btck_script_pubkey_copy, btck_script_pubkey_create,
    btck_script_pubkey_destroy, btck_script_pubkey_to_bytes,
};

use crate::{
    c_serialize,
    ffi::sealed::{AsPtr, FromMutPtr, FromPtr},
    KernelError,
};

/// Common operations for script pubkeys, implemented by both owned and borrowed types.
pub trait ScriptPubkeyExt: AsPtr<btck_ScriptPubkey> {
    /// Serializes the script to raw bytes.
    fn to_bytes(&self) -> Vec<u8> {
        c_serialize(|callback, user_data| unsafe {
            btck_script_pubkey_to_bytes(self.as_ptr(), Some(callback), user_data)
        })
        .expect("Script pubkey to_bytes should never fail")
    }
}

/// A single script pubkey containing spending conditions for a transaction output.
///
/// Script pubkeys can be created from raw script bytes or retrieved from existing
/// transaction outputs.
#[derive(Debug)]
pub struct ScriptPubkey {
    inner: *mut btck_ScriptPubkey,
}

unsafe impl Send for ScriptPubkey {}
unsafe impl Sync for ScriptPubkey {}

impl ScriptPubkey {
    pub fn new(script_bytes: &[u8]) -> Result<Self, KernelError> {
        let inner = unsafe {
            btck_script_pubkey_create(script_bytes.as_ptr() as *const c_void, script_bytes.len())
        };

        if inner.is_null() {
            Err(KernelError::Internal(
                "Failed to create ScriptPubkey from bytes".to_string(),
            ))
        } else {
            Ok(ScriptPubkey { inner })
        }
    }

    pub fn as_ref(&self) -> ScriptPubkeyRef<'_> {
        unsafe { ScriptPubkeyRef::from_ptr(self.inner as *const _) }
    }
}

impl AsPtr<btck_ScriptPubkey> for ScriptPubkey {
    fn as_ptr(&self) -> *const btck_ScriptPubkey {
        self.inner as *const _
    }
}

impl FromMutPtr<btck_ScriptPubkey> for ScriptPubkey {
    unsafe fn from_ptr(ptr: *mut btck_ScriptPubkey) -> Self {
        ScriptPubkey { inner: ptr }
    }
}

impl ScriptPubkeyExt for ScriptPubkey {}

impl Clone for ScriptPubkey {
    fn clone(&self) -> Self {
        ScriptPubkey {
            inner: unsafe { btck_script_pubkey_copy(self.inner) },
        }
    }
}

impl Drop for ScriptPubkey {
    fn drop(&mut self) {
        unsafe { btck_script_pubkey_destroy(self.inner) }
    }
}

impl TryFrom<&[u8]> for ScriptPubkey {
    type Error = KernelError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        ScriptPubkey::new(bytes)
    }
}

impl From<ScriptPubkey> for Vec<u8> {
    fn from(script: ScriptPubkey) -> Self {
        script.to_bytes()
    }
}

impl From<&ScriptPubkey> for Vec<u8> {
    fn from(script: &ScriptPubkey) -> Self {
        script.to_bytes()
    }
}

pub struct ScriptPubkeyRef<'a> {
    inner: *const btck_ScriptPubkey,
    marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for ScriptPubkeyRef<'a> {}
unsafe impl<'a> Sync for ScriptPubkeyRef<'a> {}

impl<'a> ScriptPubkeyRef<'a> {
    pub fn to_owned(&self) -> ScriptPubkey {
        ScriptPubkey {
            inner: unsafe { btck_script_pubkey_copy(self.inner) },
        }
    }
}

impl<'a> AsPtr<btck_ScriptPubkey> for ScriptPubkeyRef<'a> {
    fn as_ptr(&self) -> *const btck_ScriptPubkey {
        self.inner
    }
}

impl<'a> FromPtr<btck_ScriptPubkey> for ScriptPubkeyRef<'a> {
    unsafe fn from_ptr(ptr: *const btck_ScriptPubkey) -> Self {
        ScriptPubkeyRef {
            inner: ptr,
            marker: PhantomData,
        }
    }
}

impl<'a> ScriptPubkeyExt for ScriptPubkeyRef<'a> {}

impl<'a> From<ScriptPubkeyRef<'a>> for Vec<u8> {
    fn from(script_ref: ScriptPubkeyRef<'a>) -> Self {
        script_ref.to_bytes()
    }
}

impl<'a> From<&ScriptPubkeyRef<'a>> for Vec<u8> {
    fn from(script_ref: &ScriptPubkeyRef<'a>) -> Self {
        script_ref.to_bytes()
    }
}

impl<'a> Clone for ScriptPubkeyRef<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> Copy for ScriptPubkeyRef<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_utils::{test_owned_ffi_traits, test_ref_ffi_traits};

    test_owned_ffi_traits!(
        test_scriptpubkey_implementations,
        ScriptPubkey,
        btck_ScriptPubkey
    );
    test_ref_ffi_traits!(
        test_scriptpubkey_ref_implementations,
        ScriptPubkeyRef<'static>,
        btck_ScriptPubkey
    );

    fn create_test_script_bytes() -> Vec<u8> {
        // P2WPKH script: OP_0 <20-byte-hash>
        hex::decode("0014751e76e8199196d454941c45d1b3a323f1433bd6").unwrap()
    }

    fn create_p2pkh_script_bytes() -> Vec<u8> {
        // P2PKH script: OP_DUP OP_HASH160 <20-byte-hash> OP_EQUALVERIFY OP_CHECKSIG
        hex::decode("76a914fc25d6d5c94003bf5b0c7b640a248e2c637fcfb088ac").unwrap()
    }

    #[test]
    fn test_script_pubkey_new() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes);
        assert!(script.is_ok());
    }

    #[test]
    fn test_script_pubkey_new_empty() {
        let script = ScriptPubkey::new(&[]);
        assert!(script.is_ok());
    }

    #[test]
    fn test_script_pubkey_to_bytes() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();

        let retrieved_bytes = script.to_bytes();
        assert_eq!(script_bytes, retrieved_bytes);
    }

    #[test]
    fn test_script_pubkey_try_from_slice() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::try_from(script_bytes.as_slice());
        assert!(script.is_ok());
    }

    #[test]
    fn test_script_pubkey_into_vec() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();

        let vec: Vec<u8> = script.into();
        assert_eq!(vec, script_bytes);
    }

    #[test]
    fn test_script_pubkey_ref_into_vec() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();

        let vec: Vec<u8> = (&script).into();
        assert_eq!(vec, script_bytes);
    }

    #[test]
    fn test_script_pubkey_clone() {
        let script_bytes = create_test_script_bytes();
        let script1 = ScriptPubkey::new(&script_bytes).unwrap();
        let script2 = script1.clone();

        assert_eq!(script1.to_bytes(), script2.to_bytes());
    }

    #[test]
    fn test_script_pubkey_as_ref() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();

        let script_ref = script.as_ref();
        assert_eq!(script_ref.to_bytes(), script_bytes);
    }

    #[test]
    fn test_script_pubkey_ref_to_owned() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();
        let script_ref = script.as_ref();

        let owned = script_ref.to_owned();
        assert_eq!(owned.to_bytes(), script_bytes);
    }

    #[test]
    fn test_script_pubkey_ref_copy() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();
        let ref1 = script.as_ref();
        let ref2 = ref1;

        assert_eq!(ref1.to_bytes(), ref2.to_bytes());
    }

    #[test]
    fn test_script_pubkey_ref_clone() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();
        let ref1 = script.as_ref();
        let ref2 = ref1.clone();

        assert_eq!(ref1.to_bytes(), ref2.to_bytes());
    }

    #[test]
    fn test_script_pubkey_ref_into_vec_owned() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();
        let script_ref = script.as_ref();

        let vec: Vec<u8> = script_ref.into();
        assert_eq!(vec, script_bytes);
    }

    #[test]
    fn test_script_pubkey_ref_into_vec_borrowed() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();
        let script_ref = script.as_ref();

        let vec: Vec<u8> = (&script_ref).into();
        assert_eq!(vec, script_bytes);
    }

    #[test]
    fn test_script_pubkey_different_scripts() {
        let p2wpkh_bytes = create_test_script_bytes();
        let p2pkh_bytes = create_p2pkh_script_bytes();

        let p2wpkh = ScriptPubkey::new(&p2wpkh_bytes).unwrap();
        let p2pkh = ScriptPubkey::new(&p2pkh_bytes).unwrap();

        assert_eq!(p2wpkh.to_bytes(), p2wpkh_bytes);
        assert_eq!(p2pkh.to_bytes(), p2pkh_bytes);
        assert_ne!(p2wpkh.to_bytes(), p2pkh.to_bytes());
    }

    #[test]
    fn test_script_pubkey_from_mut_ptr() {
        let script_bytes = create_test_script_bytes();
        let script1 = ScriptPubkey::new(&script_bytes).unwrap();

        let ptr = unsafe { btck_script_pubkey_copy(script1.as_ptr()) };
        let script2 = unsafe { ScriptPubkey::from_ptr(ptr) };

        assert_eq!(script1.to_bytes(), script2.to_bytes());
    }

    #[test]
    fn test_script_pubkey_ref_from_ptr() {
        let script_bytes = create_test_script_bytes();
        let script = ScriptPubkey::new(&script_bytes).unwrap();

        let script_ref = unsafe { ScriptPubkeyRef::from_ptr(script.as_ptr()) };

        assert_eq!(script.to_bytes(), script_ref.to_bytes());
    }

    #[test]
    fn test_script_pubkey_ext_trait() {
        fn get_bytes_generic(script: &impl ScriptPubkeyExt) -> Vec<u8> {
            script.to_bytes()
        }

        let script_bytes = create_test_script_bytes();
        let owned_script = ScriptPubkey::new(&script_bytes).unwrap();
        let script_ref = owned_script.as_ref();

        assert_eq!(get_bytes_generic(&owned_script), script_bytes);
        assert_eq!(get_bytes_generic(&script_ref), script_bytes);
    }

    #[test]
    fn test_script_pubkey_large_script() {
        // Test with a larger script (e.g., a multisig script)
        let large_script = vec![0x51; 200];
        let script = ScriptPubkey::new(&large_script);
        assert!(script.is_ok());

        let retrieved = script.unwrap().to_bytes();
        assert_eq!(retrieved, large_script);
    }
}
