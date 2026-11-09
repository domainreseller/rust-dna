//! Utility: does the domain end in .tr? (no network)
//!
//! Run with `cargo run --example is_tr_tld`.

use domainnameapi::{json, DomainNameApi};

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = json!({
        "example.com.tr": dna.is_tr_tld("example.com.tr"),
        "example.com": dna.is_tr_tld("example.com")
    });

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
