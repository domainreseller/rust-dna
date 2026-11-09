//! Delete a child nameserver.
//!
//! Run with `cargo run --example delete_child_name_server`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.delete_child_name_server("example.com", "ns1.example.com");

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
