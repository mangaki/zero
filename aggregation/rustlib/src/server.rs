
use std::num::Wrapping;
use std::collections::{BTreeMap, BTreeSet};

use replace_with::*;
use x25519_dalek;
use sss_rs::wrapped_sharing::{Secret, reconstruct};

use crate::helpers::*;
use crate::types::*;

pub struct Server {
    threshold: usize,
    grad_len: usize,
    state: ServerState,
}

impl Server {
    pub fn new(threshold: usize, grad_len: usize) -> Self {
        Server { threshold, grad_len, state: ServerState::Round0(Collector::new(threshold)) }
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
                        .map(|(k, v)| (k, bincode::serialize(&v).unwrap())).collect())),
            Ok(ServerOutput::Gradient(v)) => Ok(ServerOutputSerialized::Gradient(v)),
            Err(()) => Err(())
        }
    }

    pub fn recv(&mut self, id: usize, msg: UserOutput) {
        match (&mut self.state, msg) {
            (ServerState::Round0(c), UserOutput::Round0(x, y)) => c.recv(id, (x, y)),
            (ServerState::Round1(c, _), UserOutput::Round1(x)) => c.recv(id, x),
            (ServerState::Round2(c, _, _), UserOutput::Round2(x)) => c.recv(id, x),
            (ServerState::Round3(c, _, _, _), UserOutput::Round3(x)) => c.recv(id, x),
            (ServerState::Round4(c, _, _, _), UserOutput::Round4(x)) => c.recv(id, x),
            _ => panic!()
        }
    }

    pub fn round(&mut self) -> Result<ServerOutput, ()> {
        // FIXME: unwrap()s !
        replace_with_or_abort_and_return(&mut self.state, |state| {
            match state {
                ServerState::Round0(c) => {
                    let m = c.get().unwrap();
                    let users = m.keys().cloned().collect::<Vec<usize>>();
                    let msg = users.into_iter().map(|id| {
                            (id, UserInput::Round1(m.clone()))
                        }).collect();
                    let rand_pks = m.into_iter().map(|(u, (_, k))| (u, k.into_msg())).collect();
                    (Ok(ServerOutput::Messages(msg)),
                        ServerState::Round1(Collector::new(self.threshold), rand_pks))
                },
                ServerState::Round1(c, rand_pks) => {
                    let mut maps = c.get().unwrap();
                    let users = maps.keys().cloned().collect::<Vec<usize>>();
                    let msgs = users.iter().map(|v| {
                        (*v, UserInput::Round2(maps.iter_mut().map(|(u, m)| (*u, m.remove(&v).unwrap())).collect()))
                    }).collect::<BTreeMap<usize, UserInput>>();
                    (Ok(ServerOutput::Messages(msgs)),
                        ServerState::Round2(Collector::new(self.threshold), rand_pks, users.into_iter().collect()))
                },
                ServerState::Round2(c, rand_pks, sharing_users) => {
                    let vecs = c.get().unwrap();
                    let users = vecs.keys().cloned().collect::<Vec<usize>>();
                    let msgs = users.iter().map(|u| (*u, UserInput::Round3(users.clone()))).collect();
                    (Ok(ServerOutput::Messages(msgs)),
                        ServerState::Round3(Collector::new(self.threshold), rand_pks, sharing_users, vecs.into_values().collect()))
                },
                ServerState::Round3(c, rand_pks, sharing_users, vecs) => {
                    let m = c.get().unwrap();
                    let users = m.keys().cloned().collect::<Vec<usize>>();
                    let msg = users.into_iter().map(|id| {
                            (id, UserInput::Round4(m.clone()))
                        }).collect();
                    (Ok(ServerOutput::Messages(msg)),
                        ServerState::Round4(Collector::new(self.threshold), rand_pks, sharing_users, vecs))
                },
                ServerState::Round4(c, rand_pks, sharing_users, vecs) => {
                    let mut m = c.get().unwrap();
                    let alive = m.keys().cloned().collect::<BTreeSet<usize>>();
                    let dropped = sharing_users.difference(&alive).cloned().collect::<BTreeSet<usize>>();
                    
                    let alive_shares = alive.iter().map(|u| {
                        let shares = m.iter_mut().map(|(v, m)| match m.remove(u).unwrap() {
                            RevealedShare::Seed(s) => s,
                            RevealedShare::RandSk(s) => panic!(),
                        }).collect();
                        (*u, shares)
                    }).collect::<BTreeMap<usize, Vec<Vec<u8>>>>();
                    let alive_secrets: BTreeMap<usize, Vec<u8>> = alive_shares.into_iter()
                        .map(|(u, shares)| {
                            let mut s = Secret::empty_in_memory();
                            reconstruct(&mut s, shares, true).unwrap();
                            (u, s.unwrap_vec())
                        }).collect();
                    let alive_contribution: Vec<Vec<Wrapping<i64>>> = alive_secrets.into_iter().map(|(v, seed)| {
                        scalar_mul(Wrapping(-1), vector_from_seed(seed.try_into().unwrap(), self.grad_len))
                    }).collect();
                    
                    let dropped_shares = dropped.iter().map(|u| {
                        let shares = m.iter_mut().map(|(v, m)| match m.remove(u).unwrap() {
                            RevealedShare::Seed(s) => panic!(),
                            RevealedShare::RandSk(s) => s,
                        }).collect();
                        (*u, shares)
                    }).collect::<BTreeMap<usize, Vec<Vec<u8>>>>();
                    let dropped_secrets: BTreeMap<usize, Vec<u8>> = dropped_shares.into_iter()
                        .map(|(u, shares)| {
                            let mut s = Secret::empty_in_memory();
                            reconstruct(&mut s, shares, true).unwrap();
                            (u, s.unwrap_vec())
                        }).collect();
                    let dropped_contribution: Vec<Vec<Wrapping<i64>>> = dropped_secrets.into_iter().map(|(u, secret)| {
                        let rand_sk = secret.try_into().unwrap();
                        let masks: Vec<Vec<Wrapping<i64>>> = alive.iter().map(|v| {
                            let other_rand_pk = rand_pks.get(v).unwrap();
                            let common_seed = x25519_dalek::x25519(rand_sk, other_rand_pk.clone());

                            use std::cmp::Ordering;
                            let l = match usize::cmp(v, &u) {
                                Ordering::Less => 1,
                                Ordering::Equal => 0,
                                Ordering::Greater => -1,
                            };
                            scalar_mul(Wrapping(l), vector_from_seed(common_seed, self.grad_len))
                        }).collect();
                        sum_components(masks.into_iter(), self.grad_len)
                    }).collect();

                    let res = sum_components(
                        Iterator::chain(alive_contribution.into_iter(), dropped_contribution.into_iter()).chain(vecs.into_iter()),
                        self.grad_len
                    );

                    (Ok(ServerOutput::Gradient(res)), ServerState::Done)
                },
                _ => panic!()
            }
        })
    }
}

