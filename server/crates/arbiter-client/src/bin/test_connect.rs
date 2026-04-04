use std::io::{self, Write};

use arbiter_client::ArbiterClient;
use arbiter_proto::{ClientMetadata, url::ArbiterUrl};

#[tokio::main]
async fn main() {
    println!("Testing connection to Arbiter server...");
    print!("Enter ArbiterUrl: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if let Err(err) = io::stdin().read_line(&mut input) {
        eprintln!("Failed to read input: {err}");
        return;
    }

    let input = input.trim();
    if input.is_empty() {
        eprintln!("ArbiterUrl cannot be empty");
        return;
    }

    let url = match ArbiterUrl::try_from(input) {
        Ok(url) => url,
        Err(err) => {
            eprintln!("Invalid ArbiterUrl: {err}");
            return;
        }
    };

    println!("{:#?}", url);

    let metadata = ClientMetadata {
        name: "arbiter-client test_connect".to_string(),
        description: Some("Manual connection smoke test".to_string()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
    };

    match ArbiterClient::connect(url, metadata).await {
        Ok(_) => println!("Connected and authenticated successfully."),
        Err(err) => eprintln!("Failed to connect: {:#?}", err),
    }
}
