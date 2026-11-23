//! Checks the credentials, searches a domain and lists your domains.
//!
//! ```bash
//! export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
//! cargo run --example quickstart -- example.com
//! ```

use domainnameapi::domains::DomainListRequest;
use domainnameapi::Client;
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    let client = Client::from_env()?;
    let domain = std::env::args().nth(1).unwrap_or_else(|| "example.com".to_string());

    // 1. The credentials and the balance.
    let account = client.account().get().await?;
    println!(
        "{} ({}): {} USD, {} TRY",
        account.reseller_name,
        client.environment(),
        account.usd_balance,
        account.try_balance
    );

    // 2. Is the domain available, and at what price?
    let search = client.domains().search(&domain).await?;
    if search.success {
        let info = &search.info;
        println!("{domain}: {} for {} {}", info.status, info.price, info.currency);
    } else {
        println!("{domain}: the search failed: {} {}", search.message, search.reason);
    }

    // 3. Every domain of the account; pages are fetched as the stream is read.
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(d) = domains.try_next().await? {
        println!("{:<40} {:<12} expires in {} days", d.domain_name, d.status, d.remaining_days);
    }
    Ok(())
}
