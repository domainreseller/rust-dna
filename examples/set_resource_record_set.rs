//! Make a record set hold exactly these values.
//!
//! Run with `cargo run --example set_resource_record_set`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.set_resource_record_set(
        "example.com",
        "@",
        "MX",
        ["10 mx1.example.com", "20 mx2.example.com"],
        3600,
        true,
    );

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
