
use core::ffi::c_void;
use libsodium_sys::*;
use serde_big_array::big_array;

serde_big_array::big_array! { BigArray; }

pub type Key = [u8; crypto_box_PUBLICKEYBYTES as usize];
pub type AEPublicKey = [u8; crypto_box_PUBLICKEYBYTES as usize];
pub type AESecretKey = [u8; crypto_box_SECRETKEYBYTES as usize];
pub type Nonce = [u8; crypto_box_NONCEBYTES as usize];
pub type KXPublicKey = [u8; crypto_kx_PUBLICKEYBYTES as usize];
pub type KXSecretKey = [u8; crypto_kx_SECRETKEYBYTES as usize];
pub type KXSessionKey = [u8; crypto_kx_SESSIONKEYBYTES as usize];
pub type SignPublicKey = [u8; crypto_sign_PUBLICKEYBYTES as usize];
pub type SignSecretKey = [u8; crypto_sign_SECRETKEYBYTES as usize];
pub type Signature = [u8; crypto_sign_BYTES as usize];

pub const SIGN_PUBLIC_KEY_BYTES: usize = crypto_sign_PUBLICKEYBYTES as usize;

pub fn nonce() -> Nonce {
    let mut nonce = [0; crypto_box_NONCEBYTES as usize];
    unsafe {
        randombytes_buf(nonce.as_mut_ptr() as *mut c_void, crypto_box_NONCEBYTES as usize);
    }
    nonce
}

pub fn gen_ae_keypair() -> (AEPublicKey, AESecretKey) {
    unsafe {
        let mut pk = [0; crypto_box_PUBLICKEYBYTES as usize];
        let mut sk = [0; crypto_box_PUBLICKEYBYTES as usize];
        crypto_box_keypair(pk.as_mut_ptr(), sk.as_mut_ptr());
        (pk, sk)
    }
}

pub fn crypto_wrap(m: &[u8], nonce: Nonce, pk: AEPublicKey, sk: AESecretKey) -> Result<Vec<u8>, ()> {
    let mut c = vec![0; (crypto_box_MACBYTES as usize) + m.len()];
    let res = unsafe {
        crypto_box_easy(c.as_mut_ptr(), m.as_ptr(), m.len() as u64, nonce.as_ptr(), pk.as_ptr(), sk.as_ptr())
    };

    if res == 0 { Ok(c) } else { Err(()) }
}

pub fn crypto_unwrap(c: &[u8], nonce: Nonce, pk: AEPublicKey, sk: AESecretKey) -> Result<Vec<u8>, ()> {
    let mut m = vec![0; c.len() - (crypto_box_MACBYTES as usize)];
    let res = unsafe {
        crypto_box_open_easy(m.as_mut_ptr(), c.as_ptr(), c.len() as u64, nonce.as_ptr(), pk.as_ptr(), sk.as_ptr())
    };

    if res == 0 { Ok(m) } else { Err(()) }
}

pub fn gen_key() -> Key {
    let mut k = [0; crypto_box_PUBLICKEYBYTES as usize];
    unsafe {
        crypto_secretbox_keygen(k.as_mut_ptr());
    }
    k
}

pub fn gen_nonce() -> Nonce {
    let mut nonce = [0; crypto_box_NONCEBYTES as usize];
    unsafe {
        randombytes_buf(nonce.as_mut_ptr() as *mut c_void, crypto_box_NONCEBYTES as usize);
    }
    nonce
}

pub fn crypto_secret_wrap(m: &[u8], nonce: Nonce, k: Key) -> Result<Vec<u8>, ()> {
    let mut c = vec![0; (crypto_box_MACBYTES as usize) + m.len()];
    let res = unsafe {
        crypto_secretbox_easy(c.as_mut_ptr(), m.as_ptr(), m.len() as u64, nonce.as_ptr(), k.as_ptr())
    };

    if res == 0 { Ok(c) } else { Err(()) }
}

pub fn crypto_secret_unwrap(c: &[u8], nonce: Nonce, k: Key) -> Result<Vec<u8>, ()> {
    let mut m = vec![0; c.len() - (crypto_box_MACBYTES as usize)];
    let res = unsafe {
        crypto_secretbox_open_easy(m.as_mut_ptr(), c.as_ptr(), c.len() as u64, nonce.as_ptr(), k.as_ptr())
    };

    if res == 0 { Ok(m) } else { Err(()) }
}

pub fn gen_kx_keypair() -> (KXPublicKey, KXSecretKey) {
    let mut pk = [0; crypto_kx_PUBLICKEYBYTES as usize];
    let mut sk = [0; crypto_kx_SECRETKEYBYTES as usize];
    unsafe {
        crypto_kx_keypair(pk.as_mut_ptr(), sk.as_mut_ptr());
    }
    (pk, sk)
}

pub fn kx_client_keys(client_pk: KXPublicKey, client_sk: KXSecretKey, server_pk: KXPublicKey) -> Result<(KXSessionKey, KXSessionKey), ()> {
    let mut rx = [0; crypto_kx_SESSIONKEYBYTES as usize];
    let mut tx = [0; crypto_kx_SESSIONKEYBYTES as usize];
    let res = unsafe {
        crypto_kx_client_session_keys(rx.as_mut_ptr(), tx.as_mut_ptr(), client_pk.as_ptr(), client_sk.as_ptr(), server_pk.as_ptr())
    };
    if res == 0 { Ok((rx, tx)) } else { Err(()) }
}

pub fn kx_server_keys(server_pk: KXPublicKey, server_sk: KXSecretKey, client_pk: KXPublicKey) -> Result<(KXSessionKey, KXSessionKey), ()> {
    let mut rx = [0; crypto_kx_SESSIONKEYBYTES as usize];
    let mut tx = [0; crypto_kx_SESSIONKEYBYTES as usize];
    let res = unsafe {
        crypto_kx_server_session_keys(rx.as_mut_ptr(), tx.as_mut_ptr(), server_pk.as_ptr(), server_sk.as_ptr(), client_pk.as_ptr())
    };
    if res == 0 { Ok((rx, tx)) } else { Err(()) }
}

pub fn gen_sign_keypair() -> (SignPublicKey, SignSecretKey) {
    let mut pk = [0; crypto_sign_PUBLICKEYBYTES as usize];
    let mut sk = [0; crypto_sign_SECRETKEYBYTES as usize];
    unsafe {
        crypto_sign_keypair(pk.as_mut_ptr(), sk.as_mut_ptr());
    };
    (pk, sk)
}

pub fn sign(m: &[u8], sk: &SignSecretKey) -> Signature {
    let mut sig = [0; crypto_sign_BYTES as usize];
    unsafe {
        crypto_sign_detached(sig.as_mut_ptr(), std::ptr::null_mut(), m.as_ptr(), m.len() as u64, sk.as_ptr());
    }
    sig
}

pub fn verify_signature(m: &[u8], sig: &Signature, pk: &SignPublicKey) -> Result<(), ()> {
    let res = unsafe {
        crypto_sign_verify_detached(sig.as_ptr(), m.as_ptr(), m.len() as u64, pk.as_ptr())
    };
    if res == 0 { Ok(()) } else { Err(()) }
}

