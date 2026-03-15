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

fn run_process(id: u32, sig: &str) {
    print!("process_request(id={id}, sig={sig:?}) => ");
    match auth::process_request(id, sig) {
        Ok(s) => println!("Ok({s})"),
        Err(e) => println!("Err(ProtoError::{e:?})"),
    }
}

fn main() {
    println!("=== authenticate ===");
    run(0, "ok");   // NotRegistered
    run(1, "bad");  // InvalidSignature
    run(99, "ok");  // InternalError1
    run(98, "ok");  // InternalError2
    run(1, "ok");   // success

    println!("\n=== process_request (Try chain) ===");
    run_process(0, "ok");   // NotRegistered (guard, no I/O)
    run_process(97, "ok");  // InternalError2 from load_config
    run_process(99, "ok");  // InternalError1 from get_nonce
    run_process(1, "bad");  // InvalidSignature from verify_signature
    run_process(1, "ok");   // success
}
