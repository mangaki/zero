
mod sodium_bindings;
mod helpers;
mod types;
mod user;
mod server;

use libsodium_sys::sodium_init;

use types::*;
use user::*;
use server::*;

fn main() {
    let ret = unsafe {
        sodium_init()
    };

    if ret != 0 {
        panic!("Failed to initialize cryptographic primitives.");
    }
}

