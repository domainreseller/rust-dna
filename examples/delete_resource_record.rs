//! Delete a record (removes the whole name + type set).
//!
//! Run with `cargo run --example delete_resource_record`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.delete_resource_record("example.com", "www", "192.0.2.5", "A", true);

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
