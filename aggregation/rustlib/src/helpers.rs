
use std::num::Wrapping;
use std::collections::{BTreeMap, BTreeSet};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Serialize, Deserialize};
#[macro_use]
use serde_big_array::big_array;

serde_big_array::big_array! { BigArray; }

use crate::sodium_bindings::*;

pub type KAPublicKey = [u8; 32];
pub type KASecretKey = [u8; 32];

pub trait Signable {
    fn as_message(&self) -> Vec<u8>;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Signed<T: Signable> {
    msg: T,
    #[serde(with = "BigArray")]
    sig: Signature,
}

impl<T: Signable> Signed<T> {
    pub fn wrap(msg: T, sk: &SignSecretKey) -> Signed<T> {
        let sig = sign(&msg.as_message(), sk);
        Signed {
            msg,
            sig,
        }
    }

    pub fn verify(&self, pk: &SignPublicKey) -> Result<(), ()> {
        verify_signature(&self.msg.as_message(), &self.sig, pk)
    }

    pub fn msg(&self) -> &T {
        &self.msg
    }

    pub fn into_msg(self) -> T {
        self.msg
    }
}

impl Signable for KAPublicKey {
    fn as_message(&self) -> Vec<u8> { self.to_vec() }
}

#[derive(Serialize, Deserialize)]
pub struct MaskGenShares {
    pub u: usize,
    pub v: usize,
    pub rand_sk_share: Vec<u8>,
    pub seed_share: Vec<u8>,
}

impl MaskGenShares {
    pub fn new(u: usize, v: usize, rand_sk_share: Vec<u8>, seed_share: Vec<u8>) -> Self {
        MaskGenShares { u, v, rand_sk_share, seed_share }
    }
}

#[derive(Serialize, Deserialize)]
pub enum RevealedShare {
    RandSk(Vec<u8>),
    Seed(Vec<u8>),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CryptoMsg {
    pub nonce: Nonce,
    pub c: Vec<u8>,
}

impl CryptoMsg {
    pub fn new(m: &[u8], k: Key) -> Self {
        let nonce = gen_nonce();
        CryptoMsg { nonce, c: crypto_secret_wrap(m, nonce, k).unwrap() } //TODO
    }

    pub fn unwrap(&self, k: Key) -> Result<Vec<u8>, ()> {
        crypto_secret_unwrap(&self.c, self.nonce, k)
    }
}

pub fn vector_from_seed(seed: [u8; 32], length: usize) -> Vec<Wrapping<i64>> {
    let mut noise = vec![Wrapping(0); length];
    let mut rng = ChaCha8Rng::from_seed(seed);
    rng.fill(noise.as_mut_slice());
    noise
}

pub fn sum_components<I>(v: I, n: usize) -> Vec<Wrapping<i64>>
    where I: Iterator<Item=Vec<Wrapping<i64>>>
{
    v.fold(vec![Wrapping(0); n], |acc, v| { Iterator::zip(acc.into_iter(), v.into_iter()).map(|(a, b)| a + b).collect() })
}

pub fn scalar_mul(l: Wrapping<i64>, v: Vec<Wrapping<i64>>) -> Vec<Wrapping<i64>> {
    v.into_iter().map(|x| l * x).collect()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BundledSignature {
    #[serde(with = "BigArray")]
    pub sig: Signature,
}

impl BundledSignature {
    pub fn new(sig: Signature) -> Self {
        BundledSignature { sig }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Collector<T> {
    threshold: usize,
    map: BTreeMap<usize, T>,
}

impl<T> Collector<T> {
    pub fn new(threshold: usize) -> Self {
        Collector { threshold, map: BTreeMap::new() }
    }

    pub fn recv(&mut self, id: usize, x: T) {
        // Receiving two inputs from the same user isn't a problem
        // (we just overwrite)
        self.map.insert(id, x);
    }

    pub fn get(self) -> Result<BTreeMap<usize, T>, ()> {
        if self.map.len() < self.threshold {
            Err(())
        } else {
            Ok(self.map)
        }
    }
}

