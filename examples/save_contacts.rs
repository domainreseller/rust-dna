//! Update the domain's contacts.
//!
//! Run with `cargo run --example save_contacts`.

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
    let result = dna.save_contacts("example.com", &contacts);

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
