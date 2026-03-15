mod auth;
mod db;
mod errors;

use errors::ProtoError;

fn run(id: u32, sig: &str) {
    print!("authenticate(id={id}, sig={sig:?}) => ");
    match auth::authenticate(id, sig) {
        Ok(nonce) => println!("Ok(nonce={nonce})"),
        Err(e) => match e.narrow::<errors::NotRegistered, _>() {
            Ok(_) => println!("Err(NotRegistered) — handled locally"),
            Err(remaining) => {
                let proto = ProtoError::from(remaining);
                println!("Err(ProtoError::{proto:?}) — forwarded to wire");
            }
        },
    }
}

fn main() {
    run(0, "ok");          // NotRegistered
    run(1, "bad");         // InvalidSignature
    run(99, "ok");         // Internal
    run(1, "ok");          // success
}
