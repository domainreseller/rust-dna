//! Balance in one currency (USD, TRY, EUR, GBP).
//!
//! Run with `cargo run --example get_current_balance`.

use domainnameapi::DomainNameApi;

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let result = dna.get_current_balance("USD");

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
