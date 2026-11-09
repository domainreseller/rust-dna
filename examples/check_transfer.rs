//! Check whether a domain can be transferred with an auth code.
//!
//! Run with `cargo run --example check_transfer`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.check_transfer("example.com", "Xy9#mK2$");

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
