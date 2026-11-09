//! Edit an existing record set.
//!
//! Run with `cargo run --example edit_resource_record`.

use domainnameapi::{json, DomainNameApi};

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.edit_resource_record(
        "example.com",
        "www",
        "www",
        "A",
        &json!("192.0.2.6"),
        600,
        true,
    );

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
