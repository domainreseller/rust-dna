//! Turn privacy protection on or off.
//!
//! Run with `cargo run --example modify_privacy_protection_status`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.modify_privacy_protection_status("example.com", true, "Owner request");

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
