
use std::sync::Arc;
use std::sync::Once;
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

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        let ret = unsafe {
            sodium_init()
        };

        if ret != 0 {
            panic!("Failed to initialize cryptographic primitives.");
        }
    })
}

fn general_test(
    participants: usize,
    active_per_round: [usize; 5],
    threshold: usize,
    grad_len: usize
)
{
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

    let masks = active_per_round.into_iter().map(|active| {
        let mut mask = (0..participants)
            .map(|u| if u < active { true } else { false })
            .collect::<Vec<bool>>();
        let mut rng = ChaCha8Rng::seed_from_u64(45);
        mask.shuffle(&mut rng);
        Iterator::zip(ids.iter(), mask.into_iter()).map(|(u, b)| (*u, b)).collect::<BTreeMap<usize, bool>>()
    }).collect::<Vec<_>>();

    let mut round = 0;
    let vec = loop {
        println!("Round {}; dropped: {:?}", round, masks[round].iter().filter(|(_, b)| !**b).map(|(u, _)| u).collect::<Vec<_>>());
        users.iter_mut().for_each(|u| {
            if *masks[round].get(&u.id()).unwrap() {
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

#[test]
fn simple_case() {
    setup();

    let participants = 9;
    let active_per_round = [9, 9, 9, 9, 9];
    let threshold = 5;
    let grad_len = 9;
    general_test(participants, active_per_round, threshold, grad_len);
}

#[test]
fn with_dropping_users() {
    setup();

    let participants = 13;
    let active_per_round = [12, 11, 10, 9, 8];
    let threshold = 5;
    let grad_len = 15;
    general_test(participants, active_per_round, threshold, grad_len);
}

#[test]
#[should_panic]
fn below_threshold() {
    setup();

    let participants = 9;
    let active_per_round = [9, 9, 9, 9, 4];
    let threshold = 7;
    let grad_len = 9;
    general_test(participants, active_per_round, threshold, grad_len);
}

