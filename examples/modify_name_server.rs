//! Replace the nameservers.
//!
//! Run with `cargo run --example modify_name_server`.

use domainnameapi::{json, DomainNameApi};

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.modify_name_server(
        "example.com",
        &json!(["ns1.example.com", "ns2.example.com"]),
    );

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
