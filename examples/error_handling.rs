//! Shows how to branch on errors: kinds, codes, validation errors and unknown outcomes.
//!
//! ```bash
//! export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
//! cargo run --example error_handling
//! ```

use domainnameapi::{codes, Client, ErrorKind};

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    let client = Client::from_env()?;

    // Checked before sending: nothing reaches the API.
    let e = client.domains().renew("", 0).await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidInput);
    for field in e.validation_errors() {
        println!("invalid: {field}");
    }

    // An API error.
    match client.domains().info("this-domain-is-not-in-your-account.com").await {
        Ok(info) => println!("found {}", info.domain_name),
        Err(e) if e.is_not_found() || e.code() == codes::DOMAIN_NOT_FOUND => {
            println!("not found: {} (hint: {})", e.message(), e.hint().unwrap_or("-"))
        }
        Err(e) => match e.kind() {
            ErrorKind::Auth => println!("check the credentials and the environment: {e}"),
            ErrorKind::RateLimited | ErrorKind::Server if e.is_retryable() => println!("try again later: {e}"),
            _ => {
                println!("{e}");
                if let Some(id) = e.correlation_id() {
                    println!("quote correlation ID {id} when contacting support");
                }
            }
        },
    }

    // Read-only mode blocks changes before they are sent.
    let read_only = Client::builder()
        .credentials(
            std::env::var("DNA_RESELLER_ID").unwrap_or_default(),
            std::env::var("DNA_API_KEY").unwrap_or_default(),
        )
        .environment(client.environment())
        .read_only(true)
        .build()?;
    let e = read_only.domains().lock("example.com").await.unwrap_err();
    println!("{}: {}", e.code(), e.message());
    Ok(())
}
