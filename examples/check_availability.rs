//! Availability of every domain x extension pair.
//!
//! Run with `cargo run --example check_availability`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.check_availability(["hello", "world123x0"], ["com", "net"], 1, "create");

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
