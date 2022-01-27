
use std::num::Wrapping;
use std::collections::{BTreeMap, BTreeSet};

use replace_with::*;
use x25519_dalek;
use sss_rs::wrapped_sharing::{Secret, share};

use crate::sodium_bindings::*;
use crate::helpers::*;
use crate::types::*;

fn round_0(data: &UserData<'_>) -> (OwnKeysData, (Signed<KAPublicKey>, Signed<KAPublicKey>)) {
    let (comm_pk, comm_sk) = {
        let secret = x25519_dalek::StaticSecret::new(rand_core::OsRng);
        (x25519_dalek::PublicKey::from(&secret).to_bytes(), secret.to_bytes())
    };
    let (rand_pk, rand_sk) = {
        let secret = x25519_dalek::StaticSecret::new(rand_core::OsRng);
        (x25519_dalek::PublicKey::from(&secret).to_bytes(), secret.to_bytes())
    };
    let own_keys = OwnKeysData {
        comm_pk,
        comm_sk,
        rand_pk,
        rand_sk,
    };

    (own_keys, (Signed::wrap(comm_pk, &data.sign_sk), Signed::wrap(rand_pk, &data.sign_sk)))
}

fn round_1(
    data: &UserData<'_>,
    own_keys: OwnKeysData,
    v: BTreeMap<usize, (Signed<KAPublicKey>, Signed<KAPublicKey>)>
)
    -> Result<((OwnKeysData, OthersKeysData, [u8; 32]), BTreeMap<usize, CryptoMsg>), ()>
{
    let n = v.len();
    if n < data.threshold {
        return Err(())
    }

    if !(v.iter().all(|(id, (x, y))| {
        let pk = data.others_sign_pks.get(id).unwrap();
        x.verify(pk).is_ok() && y.verify(pk).is_ok()
    })) {
        return Err(())
    }

    let comm_pks: BTreeMap<usize, KAPublicKey> = v.iter().map(|(id, (x, _))| (*id, *x.msg())).collect();
    let rand_pks: BTreeMap<usize, KAPublicKey> = v.iter().map(|(id, (_, x))| (*id, *x.msg())).collect();

    let seed = {
        let mut seed = [0; 32];
        match getrandom::getrandom(&mut seed) {
            Ok(()) => (),
            Err(_) => panic!(), //TODO
        }
        seed
    };

    //FIXME: Find an implementation that allows for higher numbers of shares !
    let rand_sk_shares = share(Secret::InMemory(own_keys.rand_sk.to_vec()), data.threshold as u8, n as u8, true).unwrap();
    let seed_shares = share(Secret::InMemory(seed.to_vec()), data.threshold as u8, n as u8, true).unwrap();

    let msgs: BTreeMap<usize, CryptoMsg> = comm_pks.iter()
        .zip(Iterator::zip(rand_sk_shares.into_iter(), seed_shares.into_iter()))
        .map(|((id, other_comm_pk), (rand_sk_share, seed_share))| {
            let common_key = x25519_dalek::x25519(own_keys.comm_sk, other_comm_pk.clone());
            let msg_struct = MaskGenShares::new(data.id, *id, rand_sk_share, seed_share);

            let msg = CryptoMsg::new(
                &bincode::serialize(&msg_struct).unwrap(),
                common_key);
            (*id, msg) //TODO
        }).collect();

    let others_keys = OthersKeysData { comm_pks, rand_pks };
    
    Ok(((own_keys, others_keys, seed), msgs))
}

fn round_2(
    data: &UserData,
    own_keys: OwnKeysData,
    others_keys: OthersKeysData,
    own_seed: [u8; 32],
    crypted_keys: BTreeMap<usize, CryptoMsg>
)
    -> Result<((OwnKeysData, OthersKeysData, [u8; 32], BTreeMap<usize, CryptoMsg>), Vec<Wrapping<i64>>), ()>
{
    let u_2: Vec<usize> = crypted_keys.keys().cloned().collect();

    if u_2.len() < data.threshold {
        return Err(())
    }

    let other_masks: Vec<Vec<Wrapping<i64>>> = u_2.into_iter().map(|v| {
        let rand_sk = own_keys.rand_sk;
        let other_rand_pk = others_keys.rand_pks.get(&v).unwrap();
        let common_seed = x25519_dalek::x25519(rand_sk, other_rand_pk.clone());

        use std::cmp::Ordering;
        let l = match usize::cmp(&v, &data.id) {
            Ordering::Less => 1,
            Ordering::Equal => 0,
            Ordering::Greater => -1,
        };
        scalar_mul(Wrapping(l), vector_from_seed(common_seed, data.grad.len()))
    }).collect();
    let own_mask = vector_from_seed(own_seed.clone(), data.grad.len());
    let sum: Vec<Wrapping<i64>> = sum_components(
        Iterator::chain(std::iter::once(data.grad.clone()), std::iter::once(own_mask))
            .chain(other_masks), data.grad.len());

    Ok(((own_keys, others_keys, own_seed, crypted_keys), sum))
}

fn round_3(
    data: &UserData,
    own_keys: OwnKeysData,
    others_keys: OthersKeysData,
    own_seed: [u8; 32],
    crypted_keys: BTreeMap<usize, CryptoMsg>,
    users: Vec<usize>
)
    -> Result<((OwnKeysData, OthersKeysData, [u8; 32], BTreeMap<usize, CryptoMsg>, BTreeSet<usize>), Signature), ()> {
    if users.len() < 3 {
        return Err(())
    }

    let alive: BTreeSet<usize> = users.into_iter().collect();

    Ok(((own_keys, others_keys, own_seed, crypted_keys, alive.clone()), sign(&bincode::serialize(&alive).unwrap(), &data.sign_sk)))
}

fn round_4(
    data: &UserData,
    own_keys: OwnKeysData,
    others_keys: OthersKeysData,
    own_seed: [u8; 32],
    crypted_keys: BTreeMap<usize, CryptoMsg>,
    alive: BTreeSet<usize>,
    signatures: BTreeMap<usize, BundledSignature>
) -> Result<((), BTreeMap<usize, RevealedShare>), ()> {
    let u_2: BTreeSet<usize> = crypted_keys.keys().cloned().collect();
    let u_4: BTreeSet<usize> = signatures.keys().cloned().collect();

    if u_4.len() < data.threshold {
        return Err(())
    }

    let signatures_ok = signatures.into_iter().all(|(v, sig)| {
        //TODO: unwrap()s
        let other_sign_pk = data.others_sign_pks.get(&v).unwrap();
        verify_signature(&bincode::serialize(&alive).unwrap(), &sig.sig, other_sign_pk).is_ok()
    });

    if !signatures_ok {
        return Err(())
    }

    let dropped: BTreeSet<usize> = BTreeSet::difference(&u_2, &alive).cloned().collect();

    let gen_shares: BTreeMap<usize, MaskGenShares> = crypted_keys.into_iter()
        .map(|(v, m)| {
            let v_comm_pk = others_keys.comm_pks.get(&v).unwrap();
            let comm_sk = own_keys.comm_sk;
            let clear_m = m.unwrap(x25519_dalek::x25519(comm_sk, v_comm_pk.clone()));
            let share: MaskGenShares = bincode::deserialize(&clear_m.unwrap()).unwrap();

            if !(share.u == v && share.v == data.id) {
                panic!() //TODO
            }

            (v, share) //TODO: unwrap()s
        }).collect();

    let revealed: BTreeMap<usize, RevealedShare> = Iterator::chain(
        alive.iter().map(|v| (*v, RevealedShare::Seed(gen_shares.get(&v).unwrap().seed_share.clone()))),
        dropped.iter().map(|v| (*v, RevealedShare::RandSk(gen_shares.get(&v).unwrap().rand_sk_share.clone())))
    ).collect();

    Ok(((), revealed))
}

pub struct User<'a> {
    data: UserData<'a>,
    state: UserState,
}

impl<'a> User<'a> {
    pub fn new(
        id: usize,
        threshold: usize,
        sign_pk: SignPublicKey,
        sign_sk: SignSecretKey,
        grad: Vec<Wrapping<i64>>,
        others_sign_pks: &'a BTreeMap<usize, SignPublicKey>
    ) -> User<'a> {
        User {
            data: UserData {
                id, threshold,
                sign_pk, sign_sk,
                grad,
                others_sign_pks,
            },
            state: UserState::Round0,
        }
    }

    pub fn id(&self) -> usize {
        self.data.id
    }

    pub fn round(&mut self, input: UserInput) -> Result<UserOutput, ()> {
        // FIXME: unwrap()s !
        replace_with_or_abort_and_return(&mut self.state, |state| { // HACK
            match (state, input) {
                (UserState::Round0, UserInput::Round0()) => {
                    let (own_keys, (comm_pk, rand_pk)) =
                        round_0(&self.data);
                    (Ok(UserOutput::Round0(comm_pk, rand_pk)),
                        UserState::Round1(own_keys))
                },
                (UserState::Round1(own_keys), UserInput::Round1(v)) => {
                    let ((own_keys, others_keys, seed), msgs) =
                        round_1(&self.data, own_keys, v).unwrap();
                    (Ok(UserOutput::Round1(msgs)),
                        UserState::Round2(own_keys, others_keys, seed))
                },
                (UserState::Round2(own_keys, others_keys, own_seed), UserInput::Round2(crypted_keys)) => {
                    let ((own_keys, others_keys, own_seed, crypted_keys), sum) =
                        round_2(&self.data, own_keys, others_keys, own_seed, crypted_keys).unwrap();
                    (Ok(UserOutput::Round2(sum)),
                        UserState::Round3(own_keys, others_keys, own_seed, crypted_keys))
                },
                (UserState::Round3(own_keys, others_keys, own_seed, crypted_keys), UserInput::Round3(users)) => {
                    let ((own_keys, others_keys, own_seed, crypted_keys, alive), sig) =
                        round_3(&self.data, own_keys, others_keys, own_seed, crypted_keys, users).unwrap();
                    (Ok(UserOutput::Round3(BundledSignature::new(sig))),
                        UserState::Round4(own_keys, others_keys, own_seed, crypted_keys, alive))
                },
                (UserState::Round4(own_keys, others_keys, own_seed, crypted_keys, alive), UserInput::Round4(signatures)) => {
                    let ((), x) =
                        round_4(&self.data, own_keys, others_keys, own_seed, crypted_keys, alive, signatures).unwrap();
                    (Ok(UserOutput::Round4(x)),
                        UserState::Done)
                },
                _ => panic!()
            }
        })
    }
}

