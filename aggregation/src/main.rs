
mod sodium_bindings;
mod helpers;

use std::num::Wrapping;
use std::collections::{BTreeMap, BTreeSet};

use rand::rngs::OsRng;
use libsodium_sys::*;
use x25519_dalek::{PublicKey as KAPublicKey, StaticSecret as KASecretKey};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sss_rs::wrapped_sharing::{Secret, share};
use serde::{Serialize, Deserialize};

use sodium_bindings::*;
use helpers::*;

#[derive(Clone)]
struct OwnKeysData {
    comm_pk: KAPublicKey,
    comm_sk: KASecretKey,
    rand_pk: KAPublicKey,
    rand_sk: KASecretKey,
}

struct OthersKeysData {
    comm_pks: BTreeMap<usize, KAPublicKey>,
    rand_pks: BTreeMap<usize, KAPublicKey>,
}

struct SharesData {
    shares: BTreeMap<usize, (Vec<u8>, Vec<u8>)>
}

enum UserState {
    Round0,
    Round1(OwnKeysData),
    Round2(OwnKeysData, OthersKeysData, [u8; 32]),
    Round3(OwnKeysData, OthersKeysData, [u8; 32], BTreeMap<usize, CryptoMsg>),
    Round4(OwnKeysData, OthersKeysData, [u8; 32], BTreeMap<usize, CryptoMsg>, BTreeSet<usize>),
}

struct TestUser<'a> {
    id: usize,
    threshold: usize,
    sign_pk: SignPublicKey,
    sign_sk: SignSecretKey,
    state: UserState,
    ratings: Vec<Wrapping<i64>>,
    others_sign_pks: &'a BTreeMap<usize, SignPublicKey>,
}

impl<'a> TestUser<'a> {
    pub fn new() -> TestUser<'a> {
        todo!()
    }

    pub fn round_0(&mut self) -> (Signed<KAPublicKey>, Signed<KAPublicKey>) {
        let (comm_pk, comm_sk) = {
            let secret = KASecretKey::new(rand_core::OsRng);
            (KAPublicKey::from(&secret), secret)
        };
        let (rand_pk, rand_sk) = {
            let secret = KASecretKey::new(rand_core::OsRng);
            (KAPublicKey::from(&secret), secret)
        };
        let data = OwnKeysData {
            comm_pk,
            comm_sk,
            rand_pk,
            rand_sk,
        };

        self.state = UserState::Round1(data);

        (Signed::wrap(comm_pk, &self.sign_sk), Signed::wrap(rand_pk, &self.sign_sk))
    }

    pub fn round_1(&mut self, v: BTreeMap<usize, (KAPublicKey, KAPublicKey)>) -> Result<BTreeMap<usize, CryptoMsg>, ()> {
        let own_keys = match &self.state {
            UserState::Round1(own_keys) => own_keys.clone(),
            _ => panic!()
        };

        let comm_pks: BTreeMap<usize, KAPublicKey> = v.iter().map(|(id, (x, _))| (*id, *x)).collect();
        let rand_pks: BTreeMap<usize, KAPublicKey> = v.iter().map(|(id, (_, x))| (*id, *x)).collect();

        let n = v.len();
        if n < self.threshold {
            return Err(())
        }

        let seed = {
            let mut seed = [0; 32];
            match getrandom::getrandom(&mut seed) {
                Ok(()) => (),
                Err(_) => panic!(), //TODO
            }
            seed
        };

        //FIXME: Find an implementation that allows for higher numbers of shares !
        let rand_sk_shares = share(Secret::InMemory(own_keys.rand_sk.to_bytes().to_vec()), self.threshold as u8, n as u8, true).unwrap();
        let seed_shares = share(Secret::InMemory(seed.to_vec()), self.threshold as u8, n as u8, true).unwrap();

        let msgs: BTreeMap<usize, CryptoMsg> = comm_pks.iter()
            .zip(Iterator::zip(rand_sk_shares.into_iter(), seed_shares.into_iter()))
            .map(|((id, other_comm_pk), (rand_sk_share, seed_share))| {
                let common_key = own_keys.comm_sk.diffie_hellman(other_comm_pk);
                let msg_struct = MaskGenShares::new(self.id, *id, rand_sk_share, seed_share);

                let msg = CryptoMsg::new(
                    &bincode::serialize(&msg_struct).unwrap(),
                    common_key.to_bytes());
                (*id, msg) //TODO
            }).collect();

        let others_keys = OthersKeysData { comm_pks, rand_pks };
        self.state = UserState::Round2(own_keys, others_keys, seed);
        
        Ok(msgs)
    }

    pub fn round_2(&mut self, crypted_keys: BTreeMap<usize, CryptoMsg>) -> Result<Vec<Wrapping<i64>>, ()> {
        let u_2: Vec<usize> = crypted_keys.keys().cloned().collect();

        if u_2.len() < self.threshold {
            return Err(())
        }

        let (own_keys, others_keys, own_seed) = match std::mem::replace(&mut self.state, UserState::Round0) { // HACK
            UserState::Round2(own_keys, others_keys, own_seed) => (own_keys, others_keys, own_seed),
            _ => panic!()
        };

        let other_masks: Vec<Vec<Wrapping<i64>>> = u_2.into_iter().map(|v| {
            let rand_sk = &own_keys.rand_sk;
            let other_rand_pk = others_keys.rand_pks.get(&v).unwrap();
            let common_seed = rand_sk.diffie_hellman(other_rand_pk);

            use std::cmp::Ordering;
            let l = match usize::cmp(&v, &self.id) {
                Ordering::Less => 1,
                Ordering::Equal => 0,
                Ordering::Greater => -1,
            };
            scalar_mul(Wrapping(l), vector_from_seed(common_seed.to_bytes(), self.ratings.len()))
        }).collect();
        let own_mask = vector_from_seed(own_seed.clone(), self.ratings.len());
        let sum: Vec<Wrapping<i64>> = sum_components(
            Iterator::chain(std::iter::once(self.ratings.clone()), std::iter::once(own_mask))
                .chain(other_masks), self.ratings.len());

        self.state = UserState::Round3(own_keys, others_keys, own_seed, crypted_keys);

        Ok(sum)
    }

    pub fn round_3(&mut self, users: Vec<usize>) -> Result<Signature, ()> {
        if users.len() < 3 {
            return Err(())
        }

        let (own_keys, others_keys, own_seed, crypted_keys) = match std::mem::replace(&mut self.state, UserState::Round0) { // HACK
            UserState::Round3(own_keys, others_keys, own_seed, crypted_keys) => (own_keys, others_keys, own_seed, crypted_keys),
            _ => panic!()
        };

        let alive: BTreeSet<usize> = users.into_iter().collect();
        self.state = UserState::Round4(own_keys, others_keys, own_seed, crypted_keys, alive.clone());

        Ok(sign(&bincode::serialize(&alive).unwrap(), &self.sign_sk))
    }

    pub fn round_4(&mut self, signatures: BTreeMap<usize, Signature>) -> Result<BTreeMap<usize, RevealedShare>, ()> {
        let (own_keys, others_keys, own_seed, crypted_keys, alive) = match std::mem::replace(&mut self.state, UserState::Round0) { // HACK
            UserState::Round4(own_keys, others_keys, own_seed, crypted_keys, alive) =>
                (own_keys, others_keys, own_seed, crypted_keys, alive),
            _ => panic!()
        };
        
        let u_2: BTreeSet<usize> = crypted_keys.keys().cloned().collect();
        let u_4: BTreeSet<usize> = signatures.keys().cloned().collect();

        if u_4.len() < self.threshold {
            return Err(())
        }

        let signatures_ok = signatures.into_iter().all(|(v, sig)| {
            //TODO: unwrap()s
            let other_sign_pk = self.others_sign_pks.get(&v).unwrap();
            verify_signature(&bincode::serialize(&alive).unwrap(), &sig, other_sign_pk).is_ok()
        });

        if !signatures_ok {
            return Err(())
        }

        let dropped: BTreeSet<usize> = BTreeSet::difference(&u_2, &alive).cloned().collect();

        let gen_shares: BTreeMap<usize, MaskGenShares> = crypted_keys.into_iter()
            .map(|(v, m)| {
                let v_comm_pk = others_keys.comm_pks.get(&v).unwrap();
                let comm_sk = &own_keys.comm_sk;
                let clear_m = m.unwrap(comm_sk.diffie_hellman(v_comm_pk).to_bytes());
                let share: MaskGenShares = bincode::deserialize(&clear_m.unwrap()).unwrap();

                if !(share.u == v && share.v == self.id) {
                    panic!() //TODO
                }

                (v, share) //TODO: unwrap()s
            }).collect();

        let revealed: BTreeMap<usize, RevealedShare> = Iterator::chain(
            alive.iter().map(|v| (*v, RevealedShare::Seed(gen_shares.get(&v).unwrap().seed_share.clone()))),
            dropped.iter().map(|v| (*v, RevealedShare::RandSk(gen_shares.get(&v).unwrap().rand_sk_share.clone())))
        ).collect();

        Ok(revealed)
    }
}

fn main() {
    let ret = unsafe {
        sodium_init()
    };

    if ret != 0 {
        panic!("Failed to initialize cryptographic primitives.");
    }
}

