//! Sends a request the SDK does not wrap, by implementing `Request` for your own type.
//!
//! ```bash
//! export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
//! cargo run --example custom_request -- example.com
//! ```

use domainnameapi::{Client, Endpoint, Query, Request, Validator};
use serde::Deserialize;

/// `GET /api/v1/domains/info`, written by hand, decoding only the fields this program needs.
struct ExpiryRequest {
    domain_name: String,
}

#[derive(Deserialize)]
struct Expiry {
    #[serde(rename = "domainName")]
    domain_name: String,
    #[serde(rename = "expirationDate")]
    expiration_date: Option<String>,
}

impl Request for ExpiryRequest {
    type Response = Expiry;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/domains/info")
    }

    fn query(&self) -> Query {
        Query::new().set_str("DomainName", &self.domain_name)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    let client = Client::from_env()?;
    let domain = std::env::args().nth(1).unwrap_or_else(|| "example.com".to_string());

    let response = client.send_with_response(&ExpiryRequest { domain_name: domain }).await?;
    let expiry = &response.value;
    println!("{} expires {}", expiry.domain_name, expiry.expiration_date.as_deref().unwrap_or("-"));
    println!("correlation ID {}, {} attempt(s), {:?}", response.correlation_id, response.attempts, response.duration);
    Ok(())
}
