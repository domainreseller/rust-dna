//! Create a child nameserver (glue record).
//!
//! Run with `cargo run --example add_child_name_server`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.add_child_name_server("example.com", "ns1.example.com", "192.0.2.10");

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
