//! Registers a domain with one contact for all four roles. This charges the balance: use OTE.
//!
//! ```bash
//! export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
//! cargo run --example register -- my-new-domain.com
//! ```

use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainRegisterWithContactsRequest};
use domainnameapi::Client;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    let client = Client::from_env()?;
    let domain = std::env::args().nth(1).expect("usage: register <domain>");

    let check = client.domains().search(&domain).await?;
    if !check.info.is_available() {
        println!("{domain} is not available: {} {}", check.info.status, check.info.reason);
        return Ok(());
    }
    println!("{domain} costs {} {}", check.info.price, check.info.currency);

    let contact = Contact {
        first_name: "Ada".into(),
        last_name: "Lovelace".into(),
        email: "ada@example.com".into(),
        address: "1 Example Street".into(),
        city: "Istanbul".into(),
        state: "Istanbul".into(),
        postal_code: "34000".into(),
        country: "TR".into(),
        phone_country_code: "90".into(),
        phone: "5551234567".into(),
        ..Default::default()
    };
    let request = DomainRegisterWithContactsRequest::new(
        &domain,
        DomainContacts::same(contact),
        ["tr.apiname.com", "eu.apiname.com"],
    )
    .years(1);

    match client.domains().register_with_contacts(request).await {
        Ok(result) if result.success => {
            println!("registered: {} (expires {:?})", result.status, result.expiration_date)
        }
        // The API answered, but refused (for example the registry rejected the domain).
        Ok(result) => println!("refused: {}", result.message),
        // A network failure on a billable request: it may have run. Check before trying again.
        Err(e) if e.is_outcome_unknown() => {
            println!("unknown outcome ({e}); checking the domain");
            let info = client.domains().info(&domain).await?;
            println!("{} is {}", info.domain_name, info.status);
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
