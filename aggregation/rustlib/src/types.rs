
use std::sync::Arc;
use std::num::Wrapping;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Serialize, Deserialize};
use serde_big_array::big_array;

use crate::sodium_bindings::*;
use crate::helpers::*;

serde_big_array::big_array! { BigArray; }

pub struct UserData {
    pub id: usize,
    pub threshold: usize,
    pub sign_pk: SignPublicKey,
    pub sign_sk: SignSecretKey,
    pub others_sign_pks: Arc<BTreeMap<usize, SignPublicKey>>,
    pub grad: Vec<Wrapping<i64>>,
}

#[derive(Serialize, Deserialize)]
pub struct OwnKeysData {
    pub comm_pk: KAPublicKey,
    pub comm_sk: KASecretKey,
    pub rand_pk: KAPublicKey,
    pub rand_sk: KASecretKey,
}

#[derive(Serialize, Deserialize)]
pub struct OthersKeysData {
    pub comm_pks: BTreeMap<usize, KAPublicKey>,
    pub rand_pks: BTreeMap<usize, KAPublicKey>,
}

// For reference on what each "round" is, see
// *Practical Secure Aggregation for Privacy-Preserving Machine Learning*,
// Bonowitz et. al. https://eprint.iacr.org/2017/281.pdf

#[derive(Serialize, Deserialize)]
pub enum UserState {
    Round0,
    Round1(OwnKeysData),
    Round2(OwnKeysData, OthersKeysData, [u8; 32]),
    Round3(OwnKeysData, OthersKeysData, [u8; 32], BTreeMap<usize, CryptoMsg>),
    Round4(OwnKeysData, OthersKeysData, [u8; 32], BTreeMap<usize, CryptoMsg>, BTreeSet<usize>),
    Done,
    Failed,
}

#[derive(Serialize, Deserialize)]
pub enum UserInput {
    Round0(),
    Round1(BTreeMap<usize, (Signed<KAPublicKey>, Signed<KAPublicKey>)>),
    Round2(BTreeMap<usize, CryptoMsg>),
    Round3(Vec<usize>),
    Round4(BTreeMap<usize, BundledSignature>),
}

#[derive(Serialize, Deserialize)]
pub enum UserOutput {
    Round0(Signed<KAPublicKey>, Signed<KAPublicKey>),
    Round1(BTreeMap<usize, CryptoMsg>),
    Round2(Vec<Wrapping<i64>>),
    Round3(BundledSignature),
    Round4(BTreeMap<usize, RevealedShare>),
}

#[derive(Serialize, Deserialize)]
pub enum ServerState {
    Round0(Collector<(Signed<KAPublicKey>, Signed<KAPublicKey>)>),
    Round1(Collector<BTreeMap<usize, CryptoMsg>>, BTreeMap<usize, KAPublicKey>),
    Round2(Collector<Vec<Wrapping<i64>>>, BTreeMap<usize, KAPublicKey>, BTreeSet<usize>),
    Round3(Collector<BundledSignature>, BTreeMap<usize, KAPublicKey>, BTreeSet<usize>, Vec<Vec<Wrapping<i64>>>, BTreeSet<usize>),
    Round4(Collector<BTreeMap<usize, RevealedShare>>, BTreeMap<usize, KAPublicKey>, BTreeSet<usize>, Vec<Vec<Wrapping<i64>>>, BTreeSet<usize>),
    Done,
    Failed,
}

#[derive(Serialize, Deserialize)]
pub enum ServerOutput {
    Messages(BTreeMap<usize, UserInput>),
    Gradient(Vec<Wrapping<i64>>),
}

pub enum ServerOutputSerialized {
    Messages(BTreeMap<usize, Vec<u8>>),
    Gradient(Vec<Wrapping<i64>>),
}

