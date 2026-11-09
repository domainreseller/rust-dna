//! Utility: complete and normalise a contact (no network).
//!
//! Run with `cargo run --example validate_contact`.

use domainnameapi::{json, DomainNameApi};

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.validate_contact(&json!({
        "FirstName": " Jo ",
        "Phone": "+1 (202) 555-0100",
        "Country": "Turkey"
    }));

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
