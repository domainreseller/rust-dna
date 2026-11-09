//! Replace one value of a record set.
//!
//! Run with `cargo run --example replace_resource_record_value`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.replace_resource_record_value(
        "example.com",
        "@",
        "A",
        "192.0.2.1",
        "192.0.2.2",
        None,
        true,
    );

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
