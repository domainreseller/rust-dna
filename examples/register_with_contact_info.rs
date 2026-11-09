//! Register a domain with contact information.
//!
//! Run with `cargo run --example register_with_contact_info`.

use domainnameapi::{json, DomainNameApi};

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let contact = json!({
        "FirstName": "Ahmet",
        "LastName": "Yilmaz",
        "Company": "",
        "EMail": "ahmet.yilmaz@example.com",
        "AddressLine1": "Ornek Sokak No: 123",
        "AddressLine2": "Daire 4",
        "AddressLine3": "",
        "City": "Istanbul",
        "Country": "TR",
        "Fax": "2121234567",
        "FaxCountryCode": "90",
        "Phone": "5551234567",
        "PhoneCountryCode": 90,
        "Type": "Contact",
        "ZipCode": "34000",
        "State": "Istanbul"
    });
    let contacts = json!({
        "Administrative": contact,
        "Billing": contact,
        "Technical": contact,
        "Registrant": contact
    });
    let result = dna.register_with_contact_info(
        "example.com",
        1,
        &contacts,
        &json!(["ns1.example.com", "ns2.example.com"]),
        true,  // registrar (theft protection) lock
        false, // privacy protection
        &json!({}),
    );

    // Same call with the php-dna defaults for the optional arguments:
    // let result = dna.register_with_contact_info_opts("example.com", 1, &contacts, &RegisterOptions::default());

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
