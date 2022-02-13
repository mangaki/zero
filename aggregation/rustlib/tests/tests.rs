
use std::sync::Arc;
use std::num::Wrapping;
use std::collections::BTreeMap;

use libsodium_sys::sodium_init;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand::seq::SliceRandom;

use aggregation::sodium_bindings::*;
use aggregation::types::*;
use aggregation::user::*;
use aggregation::server::*;

#[test]
fn main() {
    let ret = unsafe {
        sodium_init()
    };

    if ret != 0 {
        panic!("Failed to initialize cryptographic primitives.");
    }

    let participants = 8;
    let active = 6;
    let threshold = 3;
    let grad_len = 9;

    let ids = (0..participants).map(|u| 2 * u + 25).collect::<Vec<usize>>();

    let sign_keys = ids.iter().map(|u| {
        (*u, gen_sign_keypair())
    }).collect::<BTreeMap<usize, (SignPublicKey, SignSecretKey)>>();
    let sign_pks = Arc::new(sign_keys.iter().map(|(u, (pk, _))| (*u, pk.clone())).collect::<BTreeMap<usize, SignPublicKey>>());

    let mut users = sign_keys.into_iter().enumerate().map(|(i, (u, (sign_pk, sign_sk)))| {
        let vec = (0..grad_len)
            .map(|j| if (j % participants) == i { j as i64 + 1 } else { 0 })
            .map(Wrapping).collect();
        println!("user {} : {:?}", u, vec);
        User::new(u, threshold, sign_pk, sign_sk, vec, Arc::clone(&sign_pks))
    }).collect::<Vec<User>>();
    
    let mut server = Server::new(threshold, grad_len);

    let mut msgs: BTreeMap<usize, UserInput> = users.iter().map(|u| (u.id(), UserInput::Round0())).collect();

    let mask = {
        let mut mask = (0..participants)
            .map(|u| if u < active { true } else { false })
            .collect::<Vec<bool>>();
        let mut rng = ChaCha8Rng::seed_from_u64(45);
        mask.shuffle(&mut rng);
        Iterator::zip(ids.iter(), mask.into_iter()).map(|(u, b)| (*u, b)).collect::<BTreeMap<usize, bool>>()
    };

    let mut round = 0;
    let vec = loop {
        users.iter_mut().for_each(|u| {
            if round < 2 || *mask.get(&u.id()).unwrap() {
                let input = msgs.remove(&u.id()).unwrap();
                let output = u.round(input).unwrap();
                server.recv(u.id(), output);
            }
        });

        match server.round().unwrap() {
            ServerOutput::Messages(m) => {
                msgs = m
            },
            ServerOutput::Gradient(vec) => {
                break vec
            },
        }

        round += 1;
    };

    println!("{:?}", vec);
}

