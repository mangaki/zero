
use std::num::Wrapping;
use std::collections::{BTreeMap, BTreeSet};

use replace_with::*;
use x25519_dalek;
use sss_rs::wrapped_sharing::{Secret, reconstruct};
use serde_json;

use crate::helpers::*;
use crate::types::*;

// Implements the client server of *Practical Secure Aggregation
// for Privacy-Preserving Machine Learning*, Bonowitz et. al.
// https://eprint.iacr.org/2017/281.pdf
//
// See this paper for the reference on what each round does.

// AdvertiseKeys -- See Bonawitz et. al.
fn round_0(c: Collector<(Signed<KAPublicKey>, Signed<KAPublicKey>)>) -> Result<(ServerOutput, BTreeMap<usize, KAPublicKey>), ()> {
    let m = c.get()?;
    let users = m.keys().cloned().collect::<Vec<usize>>();
    let msg = users.into_iter().map(|id| {
            (id, UserInput::Round1(m.clone()))
        }).collect();
    let rand_pks = m.into_iter().map(|(u, (_, k))| (u, k.into_msg())).collect();
    Ok((ServerOutput::Messages(msg), rand_pks))
}

// ShareKeys -- See Bonawitz et. al.
fn round_1(
    c: Collector<BTreeMap<usize, CryptoMsg>>,
    rand_pks: BTreeMap<usize, KAPublicKey>
) -> Result<(ServerOutput, BTreeMap<usize, KAPublicKey>, BTreeSet<usize>), ()> {
    let mut maps = c.get()?;
    let users = maps.keys().cloned().collect::<Vec<usize>>();
    let msgs = users.iter().map(|v| {
        Ok((*v, UserInput::Round2(maps.iter_mut().map(|(u, m)| Ok((*u, m.remove(&v).ok_or(())?))).collect::<Result<_, ()>>()?)))
    }).collect::<Result<BTreeMap<usize, UserInput>, ()>>()?;
    Ok((ServerOutput::Messages(msgs), rand_pks, users.into_iter().collect()))
}

// MaskedInputCollection -- See Bonawitz et. al.
fn round_2(
    c: Collector<Vec<Wrapping<i64>>>,
    rand_pks: BTreeMap<usize, KAPublicKey>,
    sharing_users: BTreeSet<usize>
) -> Result<(ServerOutput, BTreeMap<usize, KAPublicKey>, BTreeSet<usize>, Vec<Vec<Wrapping<i64>>>, BTreeSet<usize>), ()> {
    let vecs = c.get()?;
    let users = vecs.keys().cloned().collect::<Vec<usize>>();
    let msgs = users.iter().map(|u| (*u, UserInput::Round3(users.clone()))).collect();
    Ok((ServerOutput::Messages(msgs), rand_pks, sharing_users, vecs.into_values().collect(), users.into_iter().collect()))
}

// ConsistencyCheck -- See Bonawitz et. al.
fn round_3(
    c: Collector<BundledSignature>,
    rand_pks: BTreeMap<usize, KAPublicKey>,
    sharing_users: BTreeSet<usize>,
    vecs: Vec<Vec<Wrapping<i64>>>,
    alive: BTreeSet<usize>,
) -> Result<(ServerOutput, BTreeMap<usize, KAPublicKey>, BTreeSet<usize>, Vec<Vec<Wrapping<i64>>>, BTreeSet<usize>), ()> {
    let m = c.get()?;
    let users = m.keys().cloned().collect::<Vec<usize>>();
    let msg = users.into_iter().map(|id| {
            (id, UserInput::Round4(m.clone()))
        }).collect();
    Ok((ServerOutput::Messages(msg), rand_pks, sharing_users, vecs, alive))
}

// Unmasking -- See Bonawitz et. al.
fn round_4(
    c: Collector<BTreeMap<usize, RevealedShare>>,
    rand_pks: BTreeMap<usize, KAPublicKey>,
    sharing_users: BTreeSet<usize>,
    vecs: Vec<Vec<Wrapping<i64>>>,
    alive: BTreeSet<usize>,
    vec_len: usize,
)   -> Result<(ServerOutput, ()), ()> {
    let mut m = c.get()?;
    let dropped = sharing_users.difference(&alive).cloned().collect::<BTreeSet<usize>>();
    
    let alive_shares = alive.iter().map(|u| {
        let shares = m.iter_mut().map(|(_, m)| match m.remove(u).ok_or(())? {
            RevealedShare::Seed(s) => Ok(s),
            RevealedShare::RandSk(_) => Err(()),
        }).collect::<Result<_, ()>>()?;
        Ok((*u, shares))
    }).collect::<Result<BTreeMap<usize, Vec<Vec<u8>>>, ()>>()?;
    let alive_secrets: BTreeMap<usize, Vec<u8>> = alive_shares.into_iter()
        .map(|(u, shares)| {
            let mut s = Secret::empty_in_memory();
            reconstruct(&mut s, shares, true).map_err(|_| ())?;
            Ok((u, s.try_unwrap_vec().ok_or(())?))
        }).collect::<Result<_, ()>>()?;
    let alive_contribution: Vec<Vec<Wrapping<i64>>> = alive_secrets.into_iter().map(|(_, seed)| {
        Ok(scalar_mul(Wrapping(-1), vector_from_seed(seed.try_into().map_err(|_| ())?, vec_len)))
    }).collect::<Result<_, ()>>()?;
    
    let dropped_shares = dropped.iter().map(|u| {
        let shares = m.iter_mut().map(|(_, m)| match m.remove(u).ok_or(())? {
            RevealedShare::Seed(_) => Err(()),
            RevealedShare::RandSk(s) => Ok(s),
        }).collect::<Result<_, ()>>()?;
        Ok((*u, shares))
    }).collect::<Result<BTreeMap<usize, Vec<Vec<u8>>>, ()>>()?;
    let dropped_secrets: BTreeMap<usize, Vec<u8>> = dropped_shares.into_iter()
        .map(|(u, shares)| {
            let mut s = Secret::empty_in_memory();
            reconstruct(&mut s, shares, true).map_err(|_| ())?;
            Ok((u, s.try_unwrap_vec().ok_or(())?))
        }).collect::<Result<_, ()>>()?;
    let dropped_contribution: Vec<Vec<Wrapping<i64>>> = dropped_secrets.into_iter().map(|(u, secret)| {
        let rand_sk = secret.try_into().map_err(|_| ())?;
        let masks: Vec<Vec<Wrapping<i64>>> = alive.iter().map(|v| {
            let other_rand_pk = rand_pks.get(v).ok_or(())?;
            let common_seed = x25519_dalek::x25519(rand_sk, other_rand_pk.clone());

            use std::cmp::Ordering;
            let l = match usize::cmp(v, &u) {
                Ordering::Less => 1,
                Ordering::Equal => 0,
                Ordering::Greater => -1,
            };
            Ok(scalar_mul(Wrapping(l), vector_from_seed(common_seed, vec_len)))
        }).collect::<Result<_, ()>>()?;
        Ok(sum_components(masks.into_iter(), vec_len))
    }).collect::<Result<_, ()>>()?;

    let res = sum_components(
        Iterator::chain(alive_contribution.into_iter(), dropped_contribution.into_iter()).chain(vecs.into_iter()),
        vec_len
    );

    Ok((ServerOutput::Vector(res), ()))
}

pub struct Server {
    threshold: usize,
    vec_len: usize,
    state: ServerState,
}

impl Server {
    pub fn new(threshold: usize, vec_len: usize) -> Self {
        Server { threshold, vec_len, state: ServerState::Round0(Collector::new(threshold)) }
    }

    pub fn serialize_state(&self) -> Result<String, ()> {
        serde_json::to_string(&self.state).map_err(|_| ())
    }

    pub fn recover_state(&mut self, s: &str) -> Result<(), ()> {
        self.state = serde_json::from_str(s).map_err(|_| ())?;
        Ok(())
    }

    pub fn recv_serialized(&mut self, id: usize, msg: &[u8]) -> Result<(), ()> {
        match bincode::deserialize::<UserOutput>(msg) {
            Ok(msg) => { self.recv(id, msg); Ok(()) },
            Err(_) => Err(())
        }
    }

    pub fn round_serialized(&mut self) -> Result<ServerOutputSerialized, ()> {
        match self.round() {
            Ok(ServerOutput::Messages(res)) =>
                Ok(ServerOutputSerialized::Messages(
                        res.into_iter()
                        .map(|(k, v)| Ok((k, bincode::serialize(&v).map_err(|_| ())?))).collect::<Result<_, ()>>()?)),
            Ok(ServerOutput::Vector(v)) => Ok(ServerOutputSerialized::Vector(v)),
            Err(()) => Err(())
        }
    }

    pub fn recv(&mut self, id: usize, msg: UserOutput) {
        match (&mut self.state, msg) {
            (ServerState::Round0(c), UserOutput::Round0(x, y)) => c.recv(id, (x, y)),
            (ServerState::Round1(c, _), UserOutput::Round1(x)) => c.recv(id, x),
            (ServerState::Round2(c, _, _), UserOutput::Round2(x)) => c.recv(id, x),
            (ServerState::Round3(c, _, _, _, _), UserOutput::Round3(x)) => c.recv(id, x),
            (ServerState::Round4(c, _, _, _, _), UserOutput::Round4(x)) => c.recv(id, x),
            _ => panic!()
        }
    }

    pub fn round(&mut self) -> Result<ServerOutput, ()> {
        replace_with_or_abort_and_return(&mut self.state, |state| {
            match state {
                ServerState::Round0(c) => {
                    match round_0(c) {
                        Ok((output, rand_pks)) =>
                            (Ok(output), ServerState::Round1(Collector::new(self.threshold), rand_pks)),
                        Err(()) => (Err(()), ServerState::Failed),
                    }
                },
                ServerState::Round1(c, rand_pks) => {
                    match round_1(c, rand_pks) {
                        Ok((output, rand_pks, sharing_users)) =>
                            (Ok(output), ServerState::Round2(Collector::new(self.threshold), rand_pks, sharing_users)),
                        Err(()) => (Err(()), ServerState::Failed),
                    }
                },
                ServerState::Round2(c, rand_pks, sharing_users) => {
                    match round_2(c, rand_pks, sharing_users,) {
                        Ok((output, rand_pks, sharing_users, vecs, alive)) =>
                            (Ok(output), ServerState::Round3(Collector::new(self.threshold), rand_pks, sharing_users, vecs, alive)),
                        Err(()) => (Err(()), ServerState::Failed)
                    }
                },
                ServerState::Round3(c, rand_pks, sharing_users, vecs, alive) => {
                    match round_3(c, rand_pks, sharing_users, vecs, alive) {
                        Ok((output, rand_pks, sharing_users, vecs, alive)) =>
                            (Ok(output), ServerState::Round4(Collector::new(self.threshold), rand_pks, sharing_users, vecs, alive)),
                        Err(()) => (Err(()), ServerState::Failed)
                    }
                },
                ServerState::Round4(c, rand_pks, sharing_users, vecs, alive) => {
                    match round_4(c, rand_pks, sharing_users, vecs, alive, self.vec_len) {
                        Ok((output, ())) =>
                            (Ok(output), ServerState::Done),
                        Err(()) => (Err(()), ServerState::Failed)
                    }
                },
                _ => panic!()
            }
        })
    }
}

