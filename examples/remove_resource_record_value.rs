//! Remove one value from a record set.
//!
//! Run with `cargo run --example remove_resource_record_value`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result =
        dna.remove_resource_record_value("example.com", "@", "MX", "30 mx3.example.com", true);

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
